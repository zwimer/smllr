
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::fs;
use std::collections::hash_map::Entry;

mod proxy;
use self::proxy::{FirstKBytesProxy};
use super::ID;



pub struct FileCatalog {
    catalog: HashMap<u64, FirstKBytesProxy>,
    //id_to_dupes: HashMap<ID, (u64, Option<(FirstBytes, Option<(Hash)>)>)>,
    shortcut: HashMap<ID, u64>,
}

impl ::std::fmt::Debug for FileCatalog {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        for (size, fkbp) in &self.catalog {
            writeln!(f, " {:06}b: {:?}", size, fkbp)?;
        }
        Ok(())
    }
}

impl FileCatalog {

    pub fn new() -> Self {
        FileCatalog {
            catalog: HashMap::new(),
            shortcut: HashMap::new(),
        }
    }
    /*
    fn get_size(path: &Path) -> io::Result<u64> {
        fs::File::open(path).and_then(|f| f.metadata()).map(|md| md.len())
        //let mut f = fs::File::open(path)?;
        //let md = f.metadata()?;
        //Ok(md.len())
    }

    //fn get_id(path: &Path)

    fn get_size_or_panic(path: &Path) -> u64 {
        Self::get_size(path).unwrap_or_else(|e| {
            panic!("Failed to get filesize for {:?}: {}", path, e)
        })
    }
    */

    pub fn insert(&mut self, path: &Path) {
        let md = fs::File::open(path).and_then(|f| f.metadata()).unwrap();
        let size: u64 = md.len();
        let id = ID { dev: md.dev(), inode: md.ino() };
        self.shortcut.insert(id, size);

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


