
// TODO XXX FIXME make sure .clone does Rc::clone() !!!
//  Rc uses helper functions and not methods to avoid collisions
//  Rc::foo(rc) != rc.foo()

// ughhhhhhh so we need a RefCell not an RC if we really want to subvert
//  Rust's guarantees like this
// Rc only allows mutation if the count == 1 because that's what it should do
// fuck it we'll do it not live (read: statically)



use std::collections::{HashMap};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, Read};
use std::collections::hash_map::Entry;

use super::super::ID;
use super::super::FIRST_K_BYTES as K;

use md5;


//#[derive(Debug, Hash, PartialEq, Eq, Clone)]
//struct Hash([u8;16]);
pub type Hash = [u8;16];

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FirstBytes([u8;K]);
//pub type FirstBytes = Hash;

#[derive(Clone)]
pub struct Duplicates(Vec<PathBuf>);

impl ::std::fmt::Debug for Duplicates {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        for i in &self.0 {
            write!(f, "{:?},  ", i)?;
        }
        Ok(())
    }
}

impl Duplicates {
    fn from(path: &Path) -> Self {
        Duplicates(vec![path.to_path_buf()])
    }
    fn get_path(&self) -> &Path {
        &self.0[0]
    }
    fn push(&mut self, path: &Path) {
        self.0.push(path.to_path_buf());
    }
    fn append(&mut self, othr: &mut Duplicates) {
        // need to return anything?
        self.0.append(&mut othr.0);
    }
}


/*

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

*/

// // // // // // // // // // // // // // // // // // // // //

pub enum FirstKBytesProxy {
    Delay { 
        //path: PathBuf, 
        id: ID,
        dups: Duplicates,
    },
    Thunk { 
        thunk: HashMap<FirstBytes, HashProxy>,
        shortcut: HashMap<ID, FirstBytes>,
    },
}

impl ::std::fmt::Debug for FirstKBytesProxy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "FKBProxy::")?;
        match self {
            &FirstKBytesProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            },
            &FirstKBytesProxy::Thunk { ref thunk, .. } => {
                write!(f, "Thunk: ")?;
                for (bytes, hp) in thunk {
                    let s = String::from_utf8_lossy(&bytes.0);
                    write!(f, "``")?;
                    for c in s.chars().take(3) { write!(f, "{}", c)?; }
                    write!(f, "..")?;
                    for c in s.chars().skip(29) { write!(f, "{}", c)?; }
                    write!(f, "'':  {:?}", hp)?;
                }
            },
        }
        Ok(())
    }
}

impl FirstKBytesProxy {
    pub fn new(id: ID, path: &Path) -> Self {
        FirstKBytesProxy::Delay { 
            //path: path.to_path_buf(), 
            id,
            dups: Duplicates::from(path)
        }
    }

    fn get_first_bytes(path: &Path) -> io::Result<FirstBytes> {
        // if the file is less than K bytes, the last K-n will be zeros
        let mut f = File::open(path)?;
        let mut v = [0u8; K];
        f.read(&mut v)?;
        Ok(FirstBytes(v))
        //Ok(*md5::compute(v))
    }
    //fn get_first_bytes_or_panic(path: &Path) -> FirstBytes {
    //    Self::get_first_bytes(path).unwrap()
    //}
    /*
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
    fn _get_delay(&self) -> Option<(ID, Duplicates)> {
        match self {
            &FirstKBytesProxy::Delay { ref id, ref dups } => 
                Some((id.clone(), dups.clone())),
            &FirstKBytesProxy::Thunk { .. } => None,
        }
    }
    */

    /// Transition type from a Delay to a Thunk with the introduction of a new file
    /// Preview both files and add them to the contents of the new Thunk
    fn transition(&mut self, new_id: ID, new_path: &Path) {
        // convert from a Delay to a Thunk
        // panics if new belongs in Delay.dups
        // panics if `self` is of type Thunk
        // NOTE this involves EITHER a clone of dups OR a promise-violating hack
        let (del_id, del_dups) = match *self {
            FirstKBytesProxy::Delay { id, ref mut dups } => {
                // this is a hack
                // if there are problems with Duplicates being empty, look here
                // "steal" `dups` so we don't have to clone it
                // but we can't just take it because we can't consume self
                assert!(id != new_id);
                // OPTION A: the safer but more expensive version:
                (id, dups.clone())
                // OPTION B: the possibly dangerous but more efficient one:
                //let stolen_dups = ::std::mem::replace(dups, Duplicates(vec![]));
                //(id, stolen_dups)
            },
            _ => unreachable!(),
        };
        let mut thunk = HashMap::new();
        let mut shortcut = HashMap::new();

        let new_first_bytes = Self::get_first_bytes(new_path).unwrap();
        let old_first_bytes = Self::get_first_bytes(&del_dups.0[0]).unwrap();

        shortcut.insert(new_id, new_first_bytes.clone());
        shortcut.insert(del_id, old_first_bytes.clone());

        let new_dups = Duplicates::from(new_path);
        thunk.insert(new_first_bytes, HashProxy::from(new_id, new_dups));
        thunk.insert(old_first_bytes, HashProxy::from(del_id, del_dups));

        *self = FirstKBytesProxy::Thunk { thunk, shortcut };
    }

    pub fn insert(&mut self, id: ID, path: &Path) { 
        //if let Some(ref id, ref mut dups}) = self.get_delay() { }
        match self {
            // Insert a hard link to what's already stored in Delay
            &mut FirstKBytesProxy::Delay { id: id2, ref mut dups } if id==id2 => {
                dups.push(path);
            },
            // Add another path and its first bytes if we're a Thunk
            &mut FirstKBytesProxy::Thunk { ref mut thunk, ref mut shortcut } => {
                let first_bytes = Self::get_first_bytes(path).unwrap();
                shortcut.insert(id, first_bytes.clone());
                match thunk.entry(first_bytes) {
                    // call `insert` on the underlying HashProxy
                    Entry::Occupied(mut occ_entry) => {
                        occ_entry.get_mut().insert(id, Duplicates::from(path))
                    },
                    // not there: create a new HashProxy
                    Entry::Vacant(vac_entry) => {
                        vac_entry.insert(HashProxy::new(id, path));
                    },
                }
            },
            // Add another path and its first bytes if we're a Delay
            &mut FirstKBytesProxy::Delay { .. } => {
                self.transition(id, path)
            },
        }
    }
    /*
    // must return more complex type:
    //  will have to return info for either 1 or 2 HashProxy insertions
    fn insert(&mut self, path: &Path, first_bytes: Option<FirstBytes>) 
        -> InsResults<FirstBytes>
    {
        /*
        let path_res = Self::get_first_bytes_res(path, true);

        if let Some((del_p, del_d)) = self.get_delay() {
    H       // if we're changing state from a Thunk to a Delay
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
    */
}

// // // // // // // // // // // // // // // // // // // // //

//#[derive(Debug)]
pub enum HashProxy {
    Delay { 
        //path: PathBuf, 
        id: ID,
        dups: Duplicates,
    },
    //Thunk(HashMap<Hash, Duplicates>),
    Thunk {
        thunk: HashMap<Hash, Duplicates>,
        shortcut: HashMap<ID, Hash>,
    },
}

impl HashProxy {
    fn new(id: ID, path: &Path) -> Self {
        HashProxy::Delay { id, dups: Duplicates::from(path) }
    }
    fn from(id: ID, dups: Duplicates) -> Self {
        HashProxy::Delay { id, dups, }
    }
    fn get_hash(path: &Path) -> io::Result<Hash> {
        // not buffered for now
        let mut f = File::open(path)?;
        let mut v = vec![];
        f.read_to_end(&mut v)?;
        Ok(*md5::compute(v))
    }
    fn transition(&mut self, new_id: ID, new_dups: Duplicates) {
        // convert Delay to Thunk
        let (del_id, del_dups) = match *self {
            HashProxy::Delay { id, ref mut dups } => {
                assert!(id != new_id);
                (id, dups.clone())
            },
            _ => unreachable!(),
        };
        let mut thunk = HashMap::new();
        let mut shortcut = HashMap::new();

        let new_hash = Self::get_hash(new_dups.get_path()).unwrap();
        let old_hash = Self::get_hash(del_dups.get_path()).unwrap();

        shortcut.insert(new_id, new_hash.clone());
        shortcut.insert(del_id, old_hash.clone());

        thunk.insert(new_hash, new_dups);
        thunk.insert(old_hash, del_dups);

        *self = HashProxy::Thunk { thunk, shortcut };
    }

    fn insert(&mut self, id: ID, mut dups: Duplicates) {
        match self {
            // hard link is contained in Delay: just append it
            &mut HashProxy::Delay { id: id2, dups: ref mut dups2 } if id==id2 => {
                dups2.append(&mut dups);
            },
            // just add file and its hash to the thunk
            &mut HashProxy::Thunk { ref mut thunk, ref mut shortcut } => {
                let hash = Self::get_hash(dups.get_path()).unwrap();
                shortcut.insert(id, hash.clone());
                match thunk.entry(hash) {
                    Entry::Occupied(mut occ_entry) => {
                        occ_entry.get_mut().append(&mut dups);
                    },
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(dups);
                    },
                }
            },
            // New non-link file is added from the delay stage: transition self
            &mut HashProxy::Delay { .. } => {
                self.transition(id, dups);
            }

        }
    }

    /*
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
    */

    /*
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
    fn insert_(&mut self, path: &Path) -> InsResults<Hash> {
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
    */
}

impl ::std::fmt::Debug for HashProxy {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "HashProxy::")?;
        match self {
            &HashProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            },
            &HashProxy::Thunk { ref thunk, .. } => {
                write!(f, "Thunk: ")?;
                for (hash, d) in thunk {
                    write!(f, "``{:02X}{:02X}..{:02X}{:02X}'': {:?}", 
                             hash[0], hash[1], hash[14], hash[15], d)?;
                }
            },
        }
        Ok(())
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
