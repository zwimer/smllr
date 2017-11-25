
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::fs;
use std::collections::hash_map::Entry;

use super::ID;

mod proxy;
use self::proxy::{Duplicates, FirstKBytesProxy};

mod print;

use super::vfs::{VFS, File, MetaData};


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
    pub fn new(vfs: T) -> Self {
        FileCataloger {
            catalog: HashMap::new(),
            vfs: vfs,
            //shortcut: HashMap::new(),
        }
    }

    // each Vec<Duplicates> is a vector of all the Duplicates w/ the same content
    // Each Duplicate is a vector of links that point to one inode
    pub fn get_repeats(&self) -> Vec<Vec<Duplicates>> {
        let mut all = vec![];
        for (_size, ref fkbp) in &self.catalog {
            all.append(&mut fkbp.get_repeats());
        }
        all
    }

    pub fn insert2(&mut self, file: <T as VFS>::FileIter) {
        let md = file.get_metadata().unwrap();
        let size: u64 = md.get_len();
        // TODO: clean this up
        let id = ID { 
            dev: md.get_device().unwrap().0,
            inode: md.get_inode().0 
        };
        /*
        match self.catalog.entry(size) {
            // already there: insert it into the firstkbytesproxy
            Entry::Occupied(mut occ_entry) => occ_entry.get_mut().insert(id, file),
            // not there: create a new firstkbytesproxy
            Entry::Vacant(vac_entry) => {
                vac_entry.insert(FirstKBytesProxy::new(id, file));
            }
        }
        */
    }

    // catalog a path into the catalog
    pub fn insert(&mut self, path: &Path) {
        // fetch mandatory info
        let md = fs::File::open(path).and_then(|f| f.metadata()).unwrap();
        let size: u64 = md.len();
        let id = ID {
            dev: md.dev(),
            inode: md.ino(),
        };

        match self.catalog.entry(size) {
            // already there: insert it into the firstkbytesproxy
            Entry::Occupied(mut occ_entry) => occ_entry.get_mut().insert(id, path),
            // not there: create a new firstkbytesproxy
            Entry::Vacant(vac_entry) => {
                vac_entry.insert(FirstKBytesProxy::new(id, path));
            }
        }
    }
}
