
use std::path::{Path, PathBuf};
use std::{io, env};
use std::fs::{self, DirEntry};
use std::collections::{HashSet};

use super::vfs::{VFS, RealFileSystem, Inode, DeviceId};

//const FOLLOW_SYMLINKS_DEFAULT: bool = false;

#[derive(Debug)]
pub struct DirWalker<T: VFS> {
    // files to include/exclude
    directories: Vec<PathBuf>,
    blacklist: Vec<PathBuf>,
    //blacklist_files: HashSet<PathBuf>,
    blacklist_files: HashSet<Inode>,

    //regex_whitelist: Vec<Pattern>,
    //regex_blacklist: Vec<Pattern>,
    // alternatively, the folder/regex black/whitelists could all be boxed 
    //  traits or something that implement `match` or something
    //  This is probably the OO way to do things but incurs vtables :/

    // keep track of what inodes have been seen
    // maps device id ("Device" in `stat`) to a collection of seen inodes
    //seen: HashMap<u64, HashSet<u64>>,
    //seen: HashMap<Inode, Option<Vec<u64>>>,
    //seen: HashMap<Inode, Vec<u64>>,
    seen: HashSet<Inode>, // for now
    //seen: HashMap<DeviceId
    // can we guarantee that a file will have the same device id as its parent dir?

    vfs: T,

    // options
    //follow_symlinks: bool,
}


use vfs::{File, MetaData, FileType};


impl<M, F, V> DirWalker<V> where V: VFS<FileIter=F>, F: File<MD=M>, M: MetaData {

    pub fn new(vfs: V, dirs: Vec<&Path>) -> DirWalker<V> {
        // if any paths are relative, append them to the current working dir
        // if getting the cwd fails, the whole process should abort
        let abs_paths: io::Result<Vec<_>> = dirs.into_iter().map(|dir| {
            if dir.is_absolute() {
                Ok(dir.to_owned())
            } else {
                info!("Converting `{:?}` to absolute path", dir);
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
            blacklist_files: HashSet::new(),
            seen: HashSet::new(),
            //follow_symlinks: FOLLOW_SYMLINKS_DEFAULT,
            vfs: vfs,
        }
    }

    fn should_handle_file<T: File>(&self, f: &T, dev_id: DeviceId) -> bool {
        match f.get_inode() {
            Ok(ref inode) if self.seen.contains(inode) ||
                self.blacklist_files.contains(inode) => false,
            Err(e) => {
                warn!("Failed to look up inode for {:?}: {}", f.get_path(), e);
                false
            },
            _ => true,
        }
    }

    fn should_traverse_folder<T: File>(&self, f: &T) -> bool {
        match f.get_inode() {
            Ok(ref inode) if self.seen.contains(inode) => false,
            Err(e) => {
                warn!("Failed to look up inode for {:?}: {}", f.get_path(), e);
                false
            },
            _ => {
                let p = f.get_path();
                self.blacklist.iter().any(|bl| p.starts_with(&bl)) == false
            }
        }
    }

    fn handle_file<T: File>(&mut self, f: &T, dev_id: DeviceId) {
        // register it in self.seen
        info!("\tHANDLING FILE {:?}", f.get_path());
        match f.get_inode() {
            Ok(inode) => self.seen.insert(inode),
            Err(e) => {
                warn!("Failed to get inode for {:?}: {}", f.get_path(), e);
                return
            }
        };
    }

    pub fn traverse_folder<T: File>(&mut self, f: &T) {
        // assume should_handle_folder was called
        // mutually recursive with Self::dispatch_any_file (sorry mom)
        // a complex directory structure will be mirrored with a complex stack
        //  note this is only sorta how BS does it. his isn't the call stack
        info!("\tHANDLING FOLDER {:?}", f.get_path());
        match f.get_inode() {
            Ok(inode) => self.seen.insert(inode),
            Err(e) => {
                warn!("Failed to get inode for {:?}: {}", f.get_path(), e);
                return
            }
        };
        let dev_id = match f.get_metadata().and_then(|md| md.get_device()) {
            Ok(di) => di,
            Err(e) => {
                warn!("Failed to get metadata for {:?}: {}", f.get_path(), e);
                return
            }
        };
        let contents = match self.vfs.list_dir(f.get_path()) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to list contents of dir {:?}: {}", f.get_path(), e);
                return
            },
        };
        for entry in contents {
            match entry {
                Ok(ref e) => self.dispatch_any_file(e, Some(dev_id)),
                Err(e) => warn!("Failed to identify file in dir {:?}: {}", f.get_path(), e),
            }
        }
    }

    fn dispatch_any_file<T: File>(&mut self, f: &T, dev_id: Option<DeviceId>) {
        // handle a file, traverse a directory, or follow a symlink
        match f.get_type() {
            //Ok(FileType::File) => if self.should_handle_file(f, dev_id) {
            //    self.handle_file(f, dev_id)
            //},
            Ok(FileType::File) => {
                let dev_id = match dev_id {
                    Some(id) => id,
                    None => match f.get_metadata().and_then(|md| md.get_device()) {
                        Ok(id) => id,
                        Err(e) => {
                            warn!("Couldn't get device id for {:?}: {}", f.get_path(), e);
                            return
                        },
                    },
                };
                if self.should_handle_file(f, dev_id) {
                    self.handle_file(f, dev_id)
                }
            },
            Ok(FileType::Dir) => if self.should_traverse_folder(f) {
                self.traverse_folder(f)
            },
            Ok(FileType::Symlink) => match self.vfs.read_link(f.get_path()) {
                // if this successfully points into a loop, we'll get stuck here
                // the stdlib should prevent this though
                Ok(ref f) => {
                    let tup: (&Path, V) = (f, self.vfs.clone());
                    self.dispatch_any_file(&tup, None)
                },
                Err(e) => warn!("Couldn't resolve symlink {:?}: {}", f.get_path(), e),
            },
            Ok(FileType::Other) => info!("Ignoring unknown file {:?}", f.get_path()),
            Err(e) => warn!("Failed to get type for {:?}: {}", f.get_path(), e),
        }
    }

    pub fn traverse_all(&mut self) {
        for d in self.directories.clone() { // uhhh for now
            let tup: (&Path, V) = (&d, self.vfs.clone());
            self.dispatch_any_file(&tup, None);
        }
    }

    /*
    // Note: this is suboptimal because every new element is an allocation
    pub fn traverse_one_folder(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        // return files/links in a folder
        // currently includes hidden files
        // TODO: return a set so there aren't duplicates??
        assert!(dir.exists());
        assert!(dir.is_dir());

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
    */
}

