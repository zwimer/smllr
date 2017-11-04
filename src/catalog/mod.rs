
use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

mod proxy;
use self::proxy::{FirstKBytesProxy, Hash, FirstBytes};
use super::ID;



struct FileCatalog {
    catalog: HashMap<u64, FirstKBytesProxy>,
    id_to_dupes: HashMap<ID, (u64, Option<(FirstBytes, Option<(Hash)>)>)>,


    // 
}

impl FileCatalog {

    fn get_size(path: &Path) -> io::Result<u64> {
        let mut f = fs::File::open(path)?;
        let md = f.metadata()?;
        Ok(md.len())
    }

    fn get_size_or_warn(path: &Path) -> Option<u64> {
        match Self::get_size(path) {
            Ok(n) => Some(n),
            Err(e) => {
                warn!("Failed to get filesize for {:?}: {}", path, e);
                None
            }
        }
    }

    fn insert(&mut self, path: &Path) {

    }

}

/*
 * ID MAPS
 *  FileCataloger : ID -> size
 *  FirstProxy    : ID -> FirstKBytes
 *  HASH          : ID -> Hash
 *
 */


