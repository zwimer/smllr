
use std::path::{Path, PathBuf};
use std::{io, env};
use std::fs::{self, DirEntry};
use std::collections::{HashMap, HashSet};

use super::vfs::{VFS, RealFileSystem, Inode, DeviceId};

const FOLLOW_SYMLINKS_DEFAULT: bool = false;

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

    vfs: T,

    // options
    follow_symlinks: bool,
}


use vfs::{File, MetaData, FileType};

// unfortunately can't operate on only Path or DirEntry
// It would be really inefficient to operate on Paths only
// It is impossible to resolve a path into a DirEntry
//  the only way to do so is to iterate over the path's parent's contents
//  but this can fail if the path is the root, in which case it is impossible
trait FileLookup {
    fn path(&self) -> PathBuf;
    fn id<V: VFS>(&self, v: &V) -> io::Result<Inode>;
    fn kind<V: VFS>(&self, v: &V) -> io::Result<FileType>;
}

//impl<'a, T: File> FileLookup for &'a T {
impl<T: File> FileLookup for T {
    fn path(&self) -> PathBuf {
        self.get_path()
        //<self as File>.get_path()
    }
    fn id<V: VFS>(&self, _: &V) -> io::Result<Inode> {
        self.get_inode()
    }
    fn kind<V: VFS>(&self, _: &V) -> io::Result<FileType> {
        self.get_type()
    }
}

impl FileLookup for Path {
    fn path(&self) -> PathBuf {
        self.to_owned()
    }
    fn id<V: VFS>(&self, v: &V) -> io::Result<Inode> {
        v.get_metadata(self).map(|md| md.get_inode())
    }
    fn kind<V: VFS>(&self, v: &V) -> io::Result<FileType> {
        v.get_metadata(self).map(|md| md.get_type())
    }

}

impl<M, F, V> DirWalker<V> where V: VFS<FileIter=F>, F: File<MD=M>, M: MetaData {

    pub fn new(vfs: V, dirs: Vec<&Path>) -> DirWalker<V> {
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
            blacklist_files: HashSet::new(),
            seen: HashSet::new(),
            follow_symlinks: FOLLOW_SYMLINKS_DEFAULT,
            vfs: vfs,
        }
    }

    fn should_handle_file<T: FileLookup>(&self, fl: &T) -> bool {
        match fl.id(&self.vfs) {
            Ok(ref inode) if self.seen.contains(inode) ||
                self.blacklist_files.contains(inode) => false,
            Err(e) => {
                warn!("Failed to look up inode for `{}`: `{:?}`", 
                      fl.path().display(), e);
                false
            },
            _ => true,
        }
    }

    fn should_enter_folder<T: FileLookup>(&self, fl: &T) -> bool {
        match fl.id(&self.vfs) {
            Ok(ref inode) if self.seen.contains(inode) => false,
            Err(e) => {
                warn!("Failed to look up inode for `{}`: `{:?}`", 
                      fl.path().display(), e);
                false
            },
            _ => {
                let p = fl.path();
                self.blacklist.iter().any(|bl| p.starts_with(&bl)) == false
            }
        }
    }

    /*
    fn should_enter_folder(&self, de: &F) -> bool {
        // takes a direntry
        if self.seen.contains(&de.get_inode()) {
            return false
        }
        let p = de.get_path();
        self.blacklist.iter().any(|bl| p.starts_with(&bl)) == false
    }

    fn should_enter_folder_(&self, p: &Path) -> bool {
        // takes a path
        let md = match self.vfs.get_metadata(p) {
            Ok(md) => md,
            Err(e) => {
                warn!("Couldn't get metadata for `{}`: `{:?}`", p.display(), e);
                return false
            }
        };
        if self.seen.contains(&md.get_inode()) {
            return false
        }
        self.blacklist.iter().any(|bl| p.starts_with(&bl)) == false
    }
    */

    fn handle_file(&mut self, de: &F) {
        // register it in self.seen
        match de.get_inode() {
            Ok(inode) => self.seen.insert(inode),
            Err(e) => {
                warn!("Failed to get inode for {}: {:?}", de.get_path().display(), e);
                return
            }
        };


        unimplemented!()
    }

    fn handle_folder(&mut self, de: &Path) { }

    // traverse 
    pub fn traverse_path(&mut self, dir: &Path) {
        // for f in dir
        //  
    }

    /*
    fn dispatch_any_file<T: FileLookup>(&mut self, fl: &T) {
        match fl.kind(&self.vfs) {
            Ok(FileType::File) => if self.should_handle_file(fl) {
                self.handle_file(fl)
            },
            Ok(FileType::Dir) => if self.should_enter_folder(fl) {
                self.traverse_folder_f(fl)
            },
            Ok(FileType::Symlink) => {
                match self.vfs.read_link(fl.path()) {
                    Ok(f) => self.dispatch_any_file(f),
                    Err(e) => {
                        warn!("Failed to resolve symlink `{}`: `{:?}`", 
                              fl.path().display(), e);
                    }
                };
                // TODO: how to handle
                //self.dispatch_any_file(dest)
            },
            Ok(FileType::Other) => {},
            Err(e) => warn!("Failed to get filetype for `{}`: `{:?}`", 
                            fl.path().display(), e)
        }
    }
    */

    pub fn traverse_folder_f(&mut self, f: &F) {
        // assume should_handle_folder was called
        // for direntry in F
        let contents = match self.vfs.list_dir(f.get_path()) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to list contents of dir `{}`: `{:?}`", 
                      f.get_path().display(), e);
                return
            },
        };
        for entry in contents {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to identify file in dir `{}`: `{:?}`",
                          f.get_path().display(), e);
                    continue
                }
            };
            match entry.get_type() {
                Ok(FileType::File) => if self.should_handle_file(&entry) { 
                    self.handle_file(&entry) 
                },
                Ok(FileType::Dir) => if self.should_enter_folder(&entry) {
                    self.traverse_folder_f(&entry)
                },
                Ok(FileType::Symlink) => {
                    let dest = match self.vfs.get_metadata(entry.get_path()) {
                        Ok(f) => f,
                        Err(e) => {
                            warn!("Failed to resolve symlink `{}`: `{:?}`", 
                                  entry.get_path().display(), e);
                            continue
                        }
                    };
                }
                //Ok(Ok(FileType::File)) => unimplemented!(),
                _ => unimplemented!()
            }
        }
    }

    pub fn traverse_file_f(&mut self, f: &F) {
        // assume should_handle was called
        // call handle_file() or whatever
    }

    //pub fn into_folder<P: AsRef<Path>>(&mut self, dir: &P) -> io::Result<()> {
    pub fn handle(&mut self, dir: &Path) {
        // iterate through all files we should iterate through
        // 1. for each DirEnt, check the type, then call `handle_file`
        //      check symlink destination
        // 2. for each file, add the inode to a `seen` map (to avoid duplicates)
        // 3. for each folder, check its inode for duplication,
        //    then check it against the folder blacklist,
        
        let md = match self.vfs.get_metadata(dir) {
            Ok(md) => md,
            Err(e) => {
                warn!("File `{}` doesn't seem to exist: {:?}", dir.display(), e);
                return
            }
        };
        let kind = md.get_type();
        match kind {
            FileType::Dir => self.handle_folder(dir),
            FileType::File => self.handle_file(unimplemented!()),
            FileType::Symlink => unimplemented!(),
            FileType::Other => {},
        }
        self.seen.insert(md.get_inode());
    }

    // Note: this is suboptimal because every new element is an allocation
    pub fn traverse_folder(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
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
}

