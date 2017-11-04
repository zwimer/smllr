
// TODO XXX FIXME make sure .clone does Rc::clone() !!!
//  Rc uses helper functions and not methods to avoid collisions
//  Rc::foo(rc) != rc.foo()

// ughhhhhhh so we need a RefCell not an RC if we really want to subvert
//  Rust's guarantees like this
// Rc only allows mutation if the count == 1 because that's what it should do
// fuck it we'll do it not live (read: statically)



use std::collections::{HashMap};
use std::path::{Path, PathBuf};
//use std::rc::Rc;
use std::fs::File;
//use std::cell::RefCell;
use std::io::{self, Read};

//use super::vfs::File;
use super::super::ID;
use super::super::FIRST_K_BYTES as K;

use md5;


//#[derive(Debug, Hash, PartialEq, Eq, Clone)]
//struct Hash([u8;16]);
pub type Hash = [u8;16];

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FirstBytes([u8;K]);

#[derive(Debug, Clone)]
struct Duplicates(Vec<PathBuf>);
//struct Duplicates(Rc<Vec<PathBuf>>);
//struct Duplicates(Rc<RefCell<Vec<PathBuf>>>);
//type Duplicates = Rc<Vec<PathBuf>>;

impl Duplicates {
    fn from(path: &Path) -> Self {
        Duplicates(vec![path.to_path_buf()])
    }
    fn get_path(&self) -> &Path {
        &self.0[0]
    }
    fn append(&mut self, path: &Path) {
        // need to return anything?
        self.0.push(path.to_path_buf());
    }
}



// either the current insert or the delay can fail
// on success, percolate the result of your computation back up
//  if the delay succeeded, then also percolate its key as well
// on failure, percolate the identifier for the key so it can be invalidated
// both results must be returned independent of the other

enum InsRes<T> {
    // NewSucc is by far the most common and it doesn't need its own PathBuf
    // so distinguish it from an OldSucc, which _does_ require a PathBuf
    NewSucc { val: T },
    OldSucc { path: PathBuf, val: T },
    Failure { path: PathBuf },
}

// if first insert, don't need to return anything
//  but first insert is a new(), which returns ()
// if second insert, may need to return 2 results
// if third insert, must return precisely one res
struct InsResults<T> {
    new: InsRes<T>,
    old: Option<InsRes<T>>,
}


// // // // // // // // // // // // // // // // // // // // //

#[derive(Debug)]
pub enum FirstKBytesProxy {
    Delay { path: PathBuf, dups: Duplicates },
    Thunk(HashMap<FirstBytes, HashProxy>),
}

impl FirstKBytesProxy {
    pub fn new(path: &Path) -> Self {
        FirstKBytesProxy::Delay { 
            path: path.to_path_buf(), 
            dups: Duplicates::from(path)
        }
    }

    fn get_first_bytes(path: &Path) -> io::Result<FirstBytes> {
        // if the file is less than K bytes, the last K-n will be zeros
        let mut f = File::open(path)?;
        let mut v = [0u8; K];
        f.read(&mut v)?;
        Ok(FirstBytes(v))
    }
    fn get_first_bytes_or_warn(path: &Path) -> Option<FirstBytes> {
        match Self::get_first_bytes(path) {
            Ok(k) => Some(k),
            Err(e) => {
                warn!("Failed to read first k bytes of {:?}: {}", path, e);
                None
            }
        }
    }
    fn get_first_bytes_res(path: &Path, new: bool) -> InsRes<FirstBytes> {
        match (new, Self::get_first_bytes_or_warn(path)) {
            (true, Some(k)) => InsRes::NewSucc { val: k },
            (false,Some(k)) => InsRes::OldSucc { val: k, path: path.to_path_buf() },
            (_, None) => InsRes::Failure { path: path.to_path_buf() },
        }
    }
    fn get_delay(&self) -> Option<(PathBuf, Duplicates)> {
        match self {
            &FirstKBytesProxy::Delay { ref path,ref dups } => 
                Some((path.clone(), dups.clone())),
            &FirstKBytesProxy::Thunk(_) => None,
        }
    }

    // must return more complex type:
    //  will have to return info for either 1 or 2 HashProxy insertions
    fn insert(&mut self, path: &Path, first_bytes: Option<FirstBytes>) 
        -> InsResults<FirstBytes>
    {
        /*
        let path_res = Self::get_first_bytes_res(path, true);

        if let Some((del_p, del_d)) = self.get_delay() {
            // if we're changing state from a Thunk to a Delay
            let mut hm = HashMap::new();

            if let InsRes::NewSucc { val: ref k } = path_res {
                let hp = HashProxy::new(path);
                hm.insert(k.clone(), hp);
            }

            let del_res = Self::get_first_bytes_res(&del_p, false);
            if let InsRes::OldSucc { val: ref h, .. } = del_res {
                let hp = HashProxy::new(&del_p);
                hm.insert(h.clone(), hp);
            }

            *self = FirstKBytesProxy::Thunk(hm);
            InsResults { old: Some(del_res), new: path_res }
        } else if let FirstKBytesProxy::Thunk(ref mut thunk) = *self {
            // already a thunk, need to insert
            if let InsRes::NewSucc { val: ref k } = path_res {
                //thunk.insert(h.clone(), Duplicates::from(path));
                if let Some(entry) = thunk.get_mut(k) {
                    entry.append(path, k);
                }
            }
            InsResults { old: None, new: path_res }
        } else {
            unreachable!()
        }
        */
        unimplemented!()
    }
}

// // // // // // // // // // // // // // // // // // // // //

#[derive(Debug)]
enum HashProxy {
    Delay { path: PathBuf, dups: Duplicates },
    Thunk(HashMap<Hash, Duplicates>),
}

impl HashProxy {
    fn new(path: &Path) -> Self {
        HashProxy::Delay { path: path.to_path_buf(), dups: Duplicates::from(path) }
    }

    fn get_hash(path: &Path) -> io::Result<Hash> {
        // not buffered for now
        let mut f = File::open(path)?;
        let mut v = vec![];
        f.read_to_end(&mut v)?;
        let hash: Hash = *md5::compute(v);
        Ok(hash)
    }
    fn get_hash_or_warn(path: &Path) -> Option<Hash> {
        match Self::get_hash(path) {
            Ok(h) => Some(h),
            Err(e) => {
                warn!("Failed to read file {:?}: {}", path, e);
                None
            }
        }
    }
    fn get_hash_res(path: &Path, new: bool) -> InsRes<Hash> {
        match (new, Self::get_hash_or_warn(path)) {
            (true, Some(h)) => InsRes::NewSucc { val: h },
            (false,Some(h)) => InsRes::OldSucc { val: h, path: path.to_path_buf() },
            (_, None) => InsRes::Failure { path: path.to_path_buf() },
        }
    }

    fn get_delay(&self) -> Option<(PathBuf, Duplicates)> {
        // this is a helper for `insert()`: need to capture the contents of 
        //  HashProxy::Delay and then replace *self
        match self {
            &HashProxy::Delay{ref path,ref dups} => Some((path.clone(),dups.clone())),
            &HashProxy::Thunk(_) => None
        }
    }


    /// Inserts a file into the structure. 
    /// Requires returning info on success/metadata of up to 2 elements
    /// On success, return the hash and maybe the filename
    /// On failure, return the filename
    fn insert(&mut self, path: &Path) -> InsResults<Hash> {
        let path_res = Self::get_hash_res(path, true);

        if let Some((del_p, del_d)) = self.get_delay() {
            // if we're changing state from a Thunk to a Delay
            let mut hm = HashMap::new();

            if let InsRes::NewSucc { val: ref h } = path_res {
                hm.insert(h.clone(), Duplicates::from(path));
            }

            let del_res = Self::get_hash_res(&del_p, false);
            if let InsRes::OldSucc { val: ref h, .. } = del_res {
                hm.insert(h.clone(), del_d);
            }

            *self = HashProxy::Thunk(hm);
            InsResults { old: Some(del_res), new: path_res }
        } else if let HashProxy::Thunk(ref mut thunk) = *self {
            // already a thunk, need to insert
            if let InsRes::NewSucc { val: ref h } = path_res {
                thunk.insert(h.clone(), Duplicates::from(path));
            }
            InsResults { old: None, new: path_res }
        } else {
            unreachable!("you are not smarter than the rust compiler")
        }
    }

    fn append(&mut self, hash: &Hash, path: &Path) {
        // if the ID has been seen before, then FileCataloger knows its hash
        let dups = match *self {
            HashProxy::Delay { ref mut dups, .. } => dups,
            HashProxy::Thunk(ref mut thunk) => thunk.get_mut(hash).unwrap()
        };
        dups.append(path);
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
