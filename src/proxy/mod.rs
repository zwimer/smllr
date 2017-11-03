
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
enum FirstKBytesProxy {
    Delay(Duplicates),
    Thunk(HashMap<Hash, HashProxy>),
}

impl FirstKBytesProxy {
    fn new(path: &Path) -> Self {
        FirstKBytesProxy::Delay(path.into())
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
enum HashProxy {
    Delay(Duplicates),
    Thunk(HashMap<Hash, Duplicates>),
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
    fn new(path: &Path) -> Self {
        HashProxy::Delay(path.into())
    }
    fn insert(&mut self, dup: Duplicates) {
        // if self.delay is empty, then this is the first insert
        /*match self {
            &mut HashProxy::Delay(delay) => {
                *self = HashProxy::Thunk(HashMap::new());
            },
            &mut HashProxy::Thunk(ref thunk) => {}
        };
        */
        //if let HashProxy::Delay(_) = *self {
        /*if let &HashProxy::Delay(d) = &*self {
            *self = HashProxy::Thunk(HashMap::new());
        }
        */
        let d2 = {
            match &*self {
                &HashProxy::Thunk(ref d) => Some(d.clone()),
                _ => None,
            }
        };
        match (d2, self) {
            (Some(delay), &mut HashProxy::Delay(_)) => {

            },
            (_, &mut HashProxy::Thunk(ref mut thunk)) => {

            },
            _ => unreachable!(),
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
