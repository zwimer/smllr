use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::fs;
use std::collections::hash_map::Entry;

use super::ID;

mod proxy;
use self::proxy::{Duplicates, FirstKBytesProxy};

mod print;

use super::vfs::VFS;


pub struct FileCataloger<T: VFS> {
    catalog: HashMap<u64, FirstKBytesProxy>,
    vfs: T,
    //shortcut: HashMap<ID, u64>,
    // For now, omit the shortcut. We're just using the real fs right now, so
    // a file is just a Path, which has no associated metadata.
    // In the future we could get the ID from the DirWalker for free*, but
    // for now we need to retrieve the metadata to get the ID which gives the
    // size for no extra cost. So no need to map ID to size
}

impl<T: VFS> FileCataloger<T> {
    ///Initilize the filecataloger
    pub fn new(vfs: T) -> Self {
        FileCataloger {
            catalog: HashMap::new(),
            vfs: vfs,
            //shortcut: HashMap::new(),
        }
    }

    // each Vec<Duplicates> is a vector of all the Duplicates w/ the same content
    // Each Duplicate is a vector of links that point to one inode
    /// get_repeats() returns a vector of vectors of lists of duplicates
    /// such that all duplicates in the catalog are grouped together
    pub fn get_repeats(&self) -> Vec<Vec<Duplicates>> {
        let mut all = vec![];
        // for each subgrouping (done by size), get all the list of duplicates and
        // add them to are return variable. 
        for (_size, ref fkbp) in &self.catalog {
            all.append(&mut fkbp.get_repeats());
        }
        all
    }

    /// inserts path into the catalog. 
    pub fn insert(&mut self, path: &Path) {
        // get the metadata (needed for preliminary comparision and storage)
        let md = fs::File::open(path).and_then(|f| f.metadata()).unwrap();
        let size: u64 = md.len();
        let id = ID {
            dev: md.dev(),
            inode: md.ino(),
        };
        // sort by size into the appropriate proxy
        match self.catalog.entry(size) {
            // If another file of that size has been included, insert into that proxy
            Entry::Occupied(mut occ_entry) => occ_entry.get_mut().insert(&self.vfs, id, path),
            // otherwise create a new firstkbytesproxy with path as the delayed insert. 
            Entry::Vacant(vac_entry) => {
                vac_entry.insert(FirstKBytesProxy::new(id, path));
            }
        }
    }
}
