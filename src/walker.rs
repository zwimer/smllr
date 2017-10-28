
use std::path::{Path, PathBuf};
use std::{io, env};
use std::fs::{self, DirEntry};
use std::collections::{HashMap, HashSet};

const FOLLOW_SYMLINKS_DEFAULT: bool = false;

#[derive(Debug)]
pub struct DirWalker {
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

    // options
    follow_symlinks: bool,
}


impl DirWalker {
    pub fn new(dirs: Vec<&Path>) -> DirWalker {
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
    //fn traverse_folder(&self, dir: &Path) -> io::Result<Vec<fs::DirEntry>> {
    pub fn traverse_folder(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        // return files/links in a folder
        // currently includes hidden files
        // TODO: return a set so there aren't duplicates??
        assert!(dir.is_dir());
        assert!(dir.exists());

        let contents = fs::read_dir(dir)?;

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
            match entry.file_type() {
                Ok(filetype) => {
                    // probably needs some refactoring
                    // needs more thorough testing
                    if filetype.is_symlink() {
                        // check the type of the link's target
                        // NOTE: DirEntry::metadata uses fs::symlink_metadata 
                        //  which does not follow symlinks, but
                        //  Path::metadata uses fs::metadata which does.
                        // I wonder how long it will take for this to cause a bug
                        match entry.path().metadata() {
                            Err(e) => {
                                warn!("Couldn't follow link {}.\nError: {}",
                                      entry.path().display(), e); 
                                false
                            },
                            Ok(md) => md.is_file(),
                        }
                    } else {
                        filetype.is_file()
                    }
                },
                Err(e) => {
                    warn!("Couldn't identify type of item {}.\nError: {}", 
                          entry.path().display(), e);
                    false
                }, 
            }
        }).map(|entry| entry.path()) // convert DirEntry to PathBuf (allocs!)
        .filter(|path| {
            // make sure none of them are blacklisted
            self.blacklist.iter().all(|bl| path.starts_with(bl) == false)
        }).collect();

        Ok(files)
    }
}

