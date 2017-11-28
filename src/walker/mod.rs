use std::path::{Path, PathBuf};
use std::{env, io};
use std::ffi::OsStr;
use std::collections::HashSet;
use regex::{self, Regex};

use vfs::{File, FileType, MetaData, VFS};

mod test; //include unit tests

#[derive(Debug)]
pub struct DirWalker<T: VFS> {
    // files to include/exclude
    directories: Vec<PathBuf>,
    blacklist_dirs: Vec<PathBuf>,
    blacklist_patterns: Vec<Regex>,

    // keep track of the files and folders we've seen
    // `files` will only be files, `folders` will only be directories
    // symlinks will be resolved to their targets or discarded
    files: HashSet<PathBuf>,
    folders: HashSet<PathBuf>,

    // file system being traversed
    vfs: T,
}

impl<M, F, V> DirWalker<V>
where
    V: VFS<FileIter = F>,
    F: File<MD = M>,
    M: MetaData,
{
    /// Helper function to convert relative paths to absolute paths if necessary
    /// Can panic if any paths are relative and if the current directory is unknown
    fn get_abs_paths(dirs: &[&Path]) -> Vec<PathBuf> {
        // if any paths are relative, append them to the current working dir
        // if getting the cwd fails, the whole process should abort
        let abs_paths: io::Result<Vec<PathBuf>> = dirs.into_iter()
            .map(|dir| {
                if dir.is_absolute() {
                    Ok(dir.to_path_buf())
                } else {
                    debug!("Converting `{:?}` to absolute path", dir);
                    env::current_dir().map(|cwd| cwd.join(dir))
                }
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

    /// Create a new DirWalker from a list of directories
    pub fn new<P: AsRef<Path>>(vfs: V, dirs: &[P]) -> DirWalker<V> {
        let dirs: Vec<&Path> = dirs.iter().map(|p| p.as_ref()).collect();
        let abs_paths = Self::get_abs_paths(&dirs);

        DirWalker {
            directories: abs_paths,
            blacklist_dirs: vec![],
            blacklist_patterns: vec![],
            files: HashSet::new(),
            folders: HashSet::new(),
            vfs: vfs,
        }
    }

    /// Build up a DirWalker with a list of blacklisted folders
    pub fn blacklist_folders(mut self, bl: Vec<&OsStr>) -> Self {
        let paths: Vec<_> = bl.into_iter().map(|s| Path::new(s)).collect();
        let abs_paths = Self::get_abs_paths(&paths);
        self.blacklist_dirs = abs_paths;
        self
    }

    /// Build up a DirWalker with a list of blacklisted path patterns
    pub fn blacklist_patterns(mut self, bl: Vec<&str>) -> Self {
        let regexes: Result<Vec<Regex>, regex::Error> =
            bl.into_iter().map(|s| Regex::new(s)).collect();
        let regexes = regexes.unwrap_or_else(|e| panic!("Couldn't parse regex; \nError: {}", e));
        self.blacklist_patterns = regexes;
        self
    }

    /// Determine whether a file is in scope (i.e. not seen already or blacklisted)
    fn should_handle_file(&self, path: &Path) -> bool {
        // only handle files that
        //  1) haven't been seen before and
        //  2) don't match a blacklist regex pattern
        //      NOTE: if a path is invalid unicode it will never match a pattern
        !self.files.contains(path) && {
            if let Some(path_str) = path.to_str() {
                self.blacklist_patterns
                    .iter()
                    .all(|re| !re.is_match(path_str))
            } else {
                true
            }
        }
    }

    /// Determine whether a folder is in scope(i.e. not seen already or blacklisted)
    fn should_traverse_folder(&self, path: &Path) -> bool {
        // only look into folders that
        //  1) haven't been seen before,
        //  2) don't match a folder blacklist, and
        //  3) don't match a regex pattern blacklist
        //      NOTE: again, bad unicode paths will not match any regex
        !self.folders.contains(path) && self.blacklist_dirs.iter().all(|dir| !path.starts_with(dir))
            && {
                if let Some(path_str) = path.to_str() {
                    self.blacklist_patterns
                        .iter()
                        .all(|re| !re.is_match(path_str))
                } else {
                    true
                }
            }
    }

    /// Perform operation on a file: in this case just add it to a hashset
    fn handle_file(&mut self, path: &Path) {
        // do your thing: here just add to a field of filepaths
        debug!("\tHANDLING FILE {:?}", path);
        let was_absent = self.files.insert(path.to_owned());
        assert!(was_absent);
    }

    /// Operate on a folder: iterate through its contents recursively
    pub fn traverse_folder(&mut self, path: &Path) {
        // assume should_handle_folder was called
        // mutually recursive with Self::dispatch_any_file (sorry mom)
        // a complex directory structure will be mirrored with a complex stack
        //  note this is only sorta how BS does it. his isn't the call stack

        let was_absent = self.folders.insert(path.to_owned());
        assert!(was_absent);

        let contents = match self.vfs.list_dir(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to list contents of dir {:?}: {}", path, e);
                return;
            }
        };
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
            FileType::Other => debug!("Ignoring unknown file {:?}", path),
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
