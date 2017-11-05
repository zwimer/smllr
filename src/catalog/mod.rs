
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::fs;
use std::collections::hash_map::Entry;

use super::ID;

mod proxy;
use self::proxy::{FirstKBytesProxy};

mod print;



pub struct FileCatalog {
    catalog: HashMap<u64, FirstKBytesProxy>,
    //shortcut: HashMap<ID, u64>,
    // For now, omit the shortcut. We're just using the real fs right now, so
    // a file is just a Path, which has no associated metadata.
    // In the future we could get the ID from the DirWalker for free*, but
    // for now we need to retrieve the metadata to get the ID which gives the 
    // size for no extra cost. So no need to map ID to size
}

impl FileCatalog {

    pub fn new() -> Self {
        FileCatalog {
            catalog: HashMap::new(),
            //shortcut: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: &Path) {
        // fetch mandatory info
        let md = fs::File::open(path).and_then(|f| f.metadata()).unwrap();
        let size: u64 = md.len();
        let id = ID { dev: md.dev(), inode: md.ino() };

        match self.catalog.entry(size) {
            // already there: insert it into the firstkbytesproxy
            Entry::Occupied(mut occ_entry) => {
                occ_entry.get_mut().insert(id, path)
            },
            // not there: create a new firstkbytesproxy
            Entry::Vacant(vac_entry) => {
                vac_entry.insert(FirstKBytesProxy::new(id, path));
            },
        }
    }

}

/*
 * ID MAPS
 *  FileCataloger : ID -> size
 *  FirstProxy    : ID -> FirstKBytes
 *  HASH          : ID -> Hash
 *
 */


