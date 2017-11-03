
use std::collections::{HashMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::fs::File;
use std::io::{self, Read};

//use super::vfs::File;
use super::FIRST_K_BYTES as K;

use md5;


type Hash = [u8;16];
//type Duplicates = Rc<Vec<PathBuf>>;

#[derive(Debug)]
struct Duplicates(Rc<Vec<PathBuf>>);

impl<'a> From<&'a Path> for Duplicates {
    fn from(path: &Path) -> Duplicates {
        let v = vec![path.to_path_buf()];
        let rc = Rc::new(v);
        Duplicates(rc)
    }
}

// // // // // // // // // // // // // // // // // // // // //

#[derive(Debug)]
struct FirstKBytesProxy {
    delay: Option<Duplicates>,
    thunk: HashMap<Hash, HashProxy>,
}

impl FirstKBytesProxy {
    fn new(path: &Path) -> Self {
        FirstKBytesProxy {
            delay: Some(path.into()),
            thunk: HashMap::new(),
        }
    }
    fn get_first_k_bytes(path: &Path) -> io::Result<[u8;K]> {
        // if the file is less than K bytes, the last K-n will be zeros
        let mut f = File::open(path)?;
        let mut v = [0u8; K];
        f.read(&mut v)?;
        Ok(v)
    }
}

// // // // // // // // // // // // // // // // // // // // //

#[derive(Debug)]
struct HashProxy {
    delay: Option<Duplicates>,
    thunk: HashMap<Hash, Duplicates>,
}

impl HashProxy {
    fn get_hash(path: &Path) -> io::Result<Hash> {
        // not buffered for now
        let mut f = File::open(path)?;
        let mut v = vec![];
        f.read_to_end(&mut v)?;
        let hash: Hash = *md5::compute(v);
        Ok(hash)
    }
    fn insert(&mut self, dup: Duplicates) {
        // if self.delay is empty, then this is the first insert
        if let Some(old) = self.delay.take() {

        } else {
            self.delay = Some(dup);
        }
    }
}

/*
 * Pass FileCataloger a file
 *  it clones the reference into id_to_list
 *  it inserts it into file_cataloger
 *      (size, FirstKBytesProxy)
 *          FKBP is either empty (just contains a delay) or populated
 *
 *
 *
 */
