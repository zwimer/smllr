use std::path::Path;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub use super::ID;
use vfs::{File, MetaData, VFS};

pub mod proxy;
use self::proxy::{Duplicates, FirstKBytesProxy};

mod print;

mod test; // include unit tests


pub struct FileCataloger<T: VFS> {
    catalog: HashMap<u64, FirstKBytesProxy>,
    vfs: T,
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
        }
    }

    // each Vec<Duplicates> is a vector of all the Duplicates w/ the same content
    // Each Duplicate is a vector of links that point to one inode
    /// get_repeats() returns a vector of vectors of lists of duplicates
    /// such that all duplicates in the catalog are grouped together
    pub fn get_repeats(&self) -> Vec<Duplicates> {
        let mut all = vec![];
        // for each subgrouping (done by size), get all the list of duplicates and
        // add them to are return variable.
        for fkbp in self.catalog.values() {
            all.append(&mut fkbp.get_repeats());
        }
        all
    }

    /// inserts path into the catalog.
    pub fn insert(&mut self, path: &Path) {
        // get the metadata (needed for preliminary comparision and storage)
        let file = self.vfs.get_file(path).expect("No such file");
        let md = file.get_metadata().expect("IO Error getting Metadata");
        let size: u64 = md.get_len();
        let id = ID {
            dev: md.get_device().unwrap().0,
            inode: md.get_inode().0,
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
