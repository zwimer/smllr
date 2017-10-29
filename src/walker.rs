
use std::path::{Path, PathBuf};
use std::{io, env};
use std::fs::{self, DirEntry};
use std::collections::{HashMap, HashSet};

use super::vfs::{VFS, RealFileSystem};

const FOLLOW_SYMLINKS_DEFAULT: bool = false;

#[derive(Debug)]
pub struct DirWalker<T: VFS> {
    // files to include/exclude
    directories: Vec<PathBuf>,
    blacklist: Vec<PathBuf>,
    //regex_whitelist: Vec<Pattern>,
    //regex_blacklist: Vec<Pattern>,
    // alternatively, the folder/regex black/whitelists could all be boxed 
    //  traits or something that implement `match` or something
    //  This is probably the OO way to do things but incurs vtables :/

    // keep track of what inodes have been seen
    // maps device id ("Device" in `stat`) to a collection of seen inodes
    seen: HashMap<u64, HashSet<u64>>,

    vfs: T,

    // options
    follow_symlinks: bool,
}


use vfs::{File, MetaData, FileType};

impl<M, F, T: VFS<FileIter=F>> DirWalker<T> where F: File<MD=M>, M: MetaData {
//impl<M: MetaData, F: File<MD=M>, T: VFS<FileIter=F>> DirWalker<T> {
    pub fn new(vfs: T, dirs: Vec<&Path>) -> DirWalker<T> {
        // if any paths are relative, append them to the current working dir
        // if getting the cwd fails, the whole process should abort
        let abs_paths: io::Result<Vec<_>> = dirs.into_iter().map(|dir| {
            if dir.is_absolute() {
                Ok(dir.to_owned())
            } else {
                info!("Converting `{}` to absolute path", dir.display());
                env::current_dir().map(|cwd| cwd.join(dir))
            }
        }).collect();

        let abs_paths = abs_paths.unwrap_or_else(|e| {
            panic!("Couldn't retrieve current working directory; \
            try using absolute paths or fix your terminal.\n\
            Error: {}", e)
        });

        DirWalker {
            directories: abs_paths,
            blacklist: vec![],
            seen: HashMap::new(),
            follow_symlinks: FOLLOW_SYMLINKS_DEFAULT,
            vfs: vfs,
        }
    }

    fn should_handle_file(&self, de: &DirEntry) -> bool {
        unimplemented!()
    }

    fn handle_file(&self, de: &DirEntry) {
        unimplemented!()
    }


    pub fn into_folder<P: AsRef<Path>>(&self, dir: &P) {
        // iterate through all files we should iterate through
        // 1. for each DirEnt, check the type, then call `handle_file`
        //      check symlink destination
        // 2. for each file, add the inode to a `seen` map (to avoid duplicates)
        // 3. for each folder, check its inode for duplication,
        //    then check it against the folder blacklist,

    }

    // Note: this is suboptimal because every new element is an allocation
    pub fn traverse_folder(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        // return files/links in a folder
        // currently includes hidden files
        // TODO: return a set so there aren't duplicates??
        assert!(dir.is_dir());
        assert!(dir.exists());

        let contents = self.vfs.list_dir(dir)?;

        let files = contents.filter_map(|i| {
            // check if handle points to a real object
            if let Err(e) = i { 
                warn!("Couldn't identify item in {}.\nError: {}", dir.display(), e);
                None
            } else {
                i.ok()
            }
        }).filter(|entry| {
            // check if object is a file (skip directories/links)
            let filetype = match entry.get_type() {
                Ok(FileType::Symlink) => {
                    self.vfs.get_metadata(entry.get_path()).map(|m| m.get_type())
                },
                x => x,
            };
            match filetype {
                Ok(FileType::File) => true,
                Ok(FileType::Symlink) => unreachable!(),
                Ok(FileType::Dir) | Ok(FileType::Other) => false,
                Err(e) => {
                    warn!("Couldn't identify type of file `{}`: `{:?}`", 
                          entry.get_path().display(), e);
                    false
                },
            }
        }).map(|entry| entry.get_path()) // convert DirEntry to PathBuf (allocs!)
        .filter(|path| {
            // make sure none of them are blacklisted
            self.blacklist.iter().all(|bl| path.starts_with(bl) == false)
        }).collect();

        Ok(files)
    }
}

