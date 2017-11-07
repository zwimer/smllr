
use std::path::{Path, PathBuf};
use std::{env, io};
use std::ffi::OsStr;
use std::collections::HashSet;
use regex::{self, Regex};

use super::vfs::VFS;

//const FOLLOW_SYMLINKS_DEFAULT: bool = false;

#[derive(Debug)]
pub struct DirWalker<T: VFS> {
    // Member variables holding paths to include and exclude.
    directories: Vec<PathBuf>,
    blacklist_dirs: Vec<PathBuf>,
    blacklist_patterns: Vec<Regex>,

    // Member variables to keep track of the files and folders we've seen.
    // `files` will only be files, `folders` will only be directories,
    // symlinks will be resolved to their targets or discarded.
    files: HashSet<PathBuf>,
    folders: HashSet<PathBuf>,

    // The generic represents the type of file system being traversed;
    // This allows ont to inject a virtual file system for testing or a
    // wrapper for the actuall filesystem for normal use.
    vfs: T,
}


use vfs::{File, FileType, MetaData};


impl<M, F, V> DirWalker<V>
where
    V: VFS<FileIter = F>,
    F: File<MD = M>,
    M: MetaData,
{
    /// Helper function to convert relative paths to absolute paths if necessary.
    /// Can panic if any paths are relative and if the current directory is unknown.
    fn get_abs_paths(dirs: &Vec<&Path>) -> Vec<PathBuf> {
        // If any paths are relative, append them to the current working dir.
        // If getting the cwd fails, the whole process should abort.
        let abs_paths: io::Result<Vec<PathBuf>> = dirs.into_iter()
            .map(|dir| if dir.is_absolute() {
                Ok(dir.to_path_buf())
            } else {
                info!("Converting `{:?}` to absolute path", dir);
                env::current_dir().map(|cwd| cwd.join(dir))
            })
            .collect();
        abs_paths.unwrap_or_else(|e| {
            panic!(
                "Couldn't retrieve current working directory; \
            try using absolute paths or fix your terminal.\n\
            Error: {}",
                e
            )
        })
    }

    /// Creates a new DirWalker from a list of directories.
    pub fn new<P: AsRef<OsStr>>(vfs: V, dirs: Vec<P>) -> DirWalker<V> {
        // Convert all the OsStr to absolute Paths.
        let paths: Vec<&Path> = dirs.iter().map(|p| Path::new(p).as_ref()).collect();
        let abs_paths = Self::get_abs_paths(&paths);
        // Then use that to initilize the member variables.
        DirWalker {
            directories: abs_paths,
            blacklist_dirs: vec![],
            blacklist_patterns: vec![],
            files: HashSet::new(),
            folders: HashSet::new(),
            vfs: vfs,
        }
    }

    /// Add a list of  blacklisted folders to a DirWalker. This will
    /// replace the list of blacklisted folders if called twice.
    pub fn blacklist_folders(mut self, bl: Vec<&OsStr>) -> Self {
        // Generate the absolute paths for the blacklisted directores,
        let paths = bl.into_iter().map(|s| Path::new(s)).collect();
        let abs_paths = Self::get_abs_paths(&paths);
        // then set the member variable for blacklisted files.
        self.blacklist_dirs = abs_paths;
        self
    }

    /// Adds a list of blacklist patterns (regexes of files to exclude) from Strings
    ///  to the DirWalker. Replaces the list if one already has been added.
    pub fn blacklist_patterns(mut self, bl: Vec<&str>) -> Self {
        // Convert the Strings to regexes,
        let regexes: Result<Vec<Regex>, regex::Error> =
            bl.into_iter().map(|s| Regex::new(s)).collect();
        // then test that all are valid.
        let regexes = regexes.unwrap_or_else(|e| panic!("Couldn't parse regex; \nError: {}", e));
        // If no errors, set the member variable.
        self.blacklist_patterns = regexes;
        self
    }

    /// Travese's all the folders specified, then collect all specified files
    /// into a set and return it; this kills the DirWalker.
    pub fn traverse_all(mut self) -> HashSet<PathBuf> {
        // steal directories (performance hack, ask owen)
        let directories = ::std::mem::replace(&mut self.directories, vec![]);
        for path in directories {
            self.dispatch_any_file(&path, None);
        }
        self.files
    }

    /// Check and handle teh filesystem object represented by path.
    fn dispatch_any_file(&mut self, path: &Path, filetype: Option<FileType>) {
        // Get the type of the object represented by path
        let filetype = match filetype {
            Some(ft) => ft,
            None => {
                match self.vfs.get_metadata(path) {
                    Ok(md) => md.get_type(),
                    Err(e) => {
                        warn!("Couldn't get metadata for {:?}: {}", path, e);
                        return;
                    }
                }
            }
        };
        // then check if we should handle the object, then if we should,
        // either handle the file, traverse the folder, or resolve the symlink and
        // dispatch_any_file what it resolves to.
        match filetype {
            FileType::File => {
                if self.should_handle_file(path) {
                    self.handle_file(path)
                }
            }
            FileType::Dir => {
                if self.should_traverse_folder(path) {
                    self.traverse_folder(path)
                }
            }
            FileType::Symlink => {
                match self.vfs.read_link(path) {
                    Ok(ref f) => self.dispatch_any_file(f, None),
                    Err(e) => warn!("Couldn't resolve symlink {:?}: {}", path, e),
                }
            }
            FileType::Other => info!("Ignoring unknown file {:?}", path),
        }
    }
    /// Determine if a file (represented by a path) should be handled
    /// (i.e. not seen already or blacklisted).
    fn should_handle_file(&self, path: &Path) -> bool {
        // only handle files that
        //  1) haven't been seen before and
        //  2) don't match a blacklist regex pattern
        //      NOTE: if a path is invalid unicode it will never match a pattern
        self.files.contains(path) == false &&
            {
                if let Some(path_str) = path.to_str() {
                    self.blacklist_patterns.iter().all(
                        |re| !re.is_match(path_str),
                    )
                } else {
                    true
                }
            }
    }

    /// Determine if a folder (represented by a path) should be handled
    /// (i.e. not seen already or blacklisted).
    fn should_traverse_folder(&self, path: &Path) -> bool {
        // only look into folders that
        //  1) haven't been seen before,
        //  2) don't match a folder blacklist, and
        //  3) don't match a regex pattern blacklist
        //      NOTE: again, bad unicode paths will not match any regex
        self.folders.contains(path) == false &&
            self.blacklist_dirs.iter().all(|dir| !path.starts_with(dir)) &&
            {
                if let Some(path_str) = path.to_str() {
                    self.blacklist_patterns.iter().all(
                        |re| !re.is_match(path_str),
                    )
                } else {
                    true
                }
            }
        }
    }

    /// Operate on a file: add file path to the HashSet.
    fn handle_file(&mut self, path: &Path) {
        info!("\tHANDLING FILE {:?}", path);
        let was_absent = self.files.insert(path.to_owned());
        assert!(was_absent);
    }

    /// Operate on a folder: iterate through its contents recursively
    /// and add it to the HashSet.
    pub fn traverse_folder(&mut self, path: &Path) {
        // mutually recursive with Self::dispatch_any_file
        // a complex directory structure will be mirrored with a complex stack

        // If called, must not have have been added to the HashSet; else fail.
        let was_absent = self.folders.insert(path.to_owned());
        assert!(was_absent);

        // Get the contents of the folder,
        let contents = match self.vfs.list_dir(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to list contents of dir {:?}: {}", path, e);
                return;
            }
        };
        // then process (dispatch_any_file) every file in the path.
        // note that in linux, directories are files.
        for entry in contents {
            match entry {
                Ok(ref e) => self.dispatch_any_file(&e.get_path(), e.get_type().ok()),
                Err(e) => warn!("Failed to identify file in dir {:?}: {}", path, e),
            }
        }
    }

    /// Check and possibly handle any filesystem object
    fn dispatch_any_file(&mut self, path: &Path, filetype: Option<FileType>) {
        // handle a file, traverse a directory, or follow a symlink
        let filetype = match filetype {
            Some(ft) => ft,
            None => match self.vfs.get_metadata(path) {
                Ok(md) => md.get_type(),
                Err(e) => {
                    warn!("Couldn't get metadata for {:?}: {}", path, e);
                    return;
                }
            },
        };
        match filetype {
            FileType::File => if self.should_handle_file(path) {
                self.handle_file(path)
            },
            FileType::Dir => if self.should_traverse_folder(path) {
                self.traverse_folder(path)
            },
            FileType::Symlink => match self.vfs.read_link(path) {
                Ok(ref f) => self.dispatch_any_file(f, None),
                Err(e) => warn!("Couldn't resolve symlink {:?}: {}", path, e),
            },
            FileType::Other => info!("Ignoring unknown file {:?}", path),
        }
    }

    /// Collect all specified files into a set; this kills the DirWalker
    pub fn traverse_all(mut self) -> HashSet<PathBuf> {
        // steal directories (performance hack, ask owen)
        let directories = ::std::mem::replace(&mut self.directories, vec![]);
        for path in directories {
            self.dispatch_any_file(&path, None);
        }
        self.files
    }
}
