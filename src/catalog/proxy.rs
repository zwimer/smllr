
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, Read};
use std::collections::hash_map::Entry;

use super::super::ID;
use super::super::FIRST_K_BYTES as K;

use md5;

// helper types

// the type md5::compute() derefs to
pub type Hash = [u8; 16];

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FirstBytes(pub(super) [u8; K]);

#[derive(Clone)]
pub struct Duplicates(pub(super) Vec<PathBuf>);


impl Duplicates {
    // convert from a path
    fn from(path: &Path) -> Self {
        Duplicates(vec![path.to_path_buf()])
    }
    // convert to a path
    fn get_path(&self) -> &Path {
        &self.0[0]
    }
    // add one element
    fn push(&mut self, path: &Path) {
        self.0.push(path.to_path_buf());
    }
    // add all elements in another object
    fn append(&mut self, mut othr: Duplicates) {
        self.0.append(&mut othr.0);
    }
}


// // // // // // // // // // // // // // // // // // // // //

pub enum FirstKBytesProxy {
    // in the first state there is one file
    // don't look up its first k bytes unless it has the same size as another
    Delay { id: ID, dups: Duplicates },
    // after 2 files with the first k bytes have been found, store them
    // also maintain a shortcut for looking up their values by their id
    Thunk {
        thunk: HashMap<FirstBytes, HashProxy>,
        shortcut: HashMap<ID, FirstBytes>,
    },
}


impl FirstKBytesProxy {
    pub fn new(id: ID, path: &Path) -> Self {
        FirstKBytesProxy::Delay {
            id,
            dups: Duplicates::from(path),
        }
    }

    fn get_first_bytes(path: &Path) -> io::Result<FirstBytes> {
        // if the file is less than K bytes, the last K-n will be zeros
        let mut f = File::open(path)?;
        let mut v = [0u8; K];
        f.read(&mut v)?;
        Ok(FirstBytes(v)) // FirstBytes
                          //Ok(*md5::compute(v))  // Hash of FirstBytes
    }

    // identify all repeats under this node
    pub(super) fn get_repeats(&self) -> Vec<Vec<Duplicates>> {
        match self {
            &FirstKBytesProxy::Delay { .. } => vec![],
            &FirstKBytesProxy::Thunk { ref thunk, .. } => {
                thunk.iter().fold(vec![], |mut acc, (_fb, hp)| {
                    acc.append(&mut hp.get_repeats());
                    acc
                })
            }
        }
    }

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
                // OPTION A: the safer but more expensive version:
                (id, dups.clone())
                // OPTION B: the possibly dangerous but more efficient one:
                //let stolen_dups = ::std::mem::replace(dups, Duplicates(vec![]));
                //(id, stolen_dups)
            }
            _ => unreachable!(),
        };
        assert!(new_id != del_id);
        let mut thunk = HashMap::new();
        let mut shortcut = HashMap::new();

        let new_first_bytes = Self::get_first_bytes(new_path).unwrap();
        let old_first_bytes = Self::get_first_bytes(&del_dups.0[0]).unwrap();

        shortcut.insert(new_id, new_first_bytes.clone());
        shortcut.insert(del_id, old_first_bytes.clone());

        let new_dups = Duplicates::from(new_path);
        if new_first_bytes == old_first_bytes {
            let mut hp = HashProxy::new(del_id, del_dups);
            hp.insert(new_id, new_dups);
            thunk.insert(old_first_bytes, hp);
        } else {
            thunk.insert(new_first_bytes, HashProxy::new(new_id, new_dups));
            thunk.insert(old_first_bytes, HashProxy::new(del_id, del_dups));
        }

        *self = FirstKBytesProxy::Thunk { thunk, shortcut };
    }

    pub fn insert(&mut self, id: ID, path: &Path) {
        match self {
            // Insert a hard link to what's already stored in Delay
            &mut FirstKBytesProxy::Delay {
                id: id2,
                ref mut dups,
            } if id == id2 =>
            {
                dups.push(path);
            }
            // Add another path and its first bytes if we're a Thunk
            &mut FirstKBytesProxy::Thunk {
                ref mut thunk,
                ref mut shortcut,
            } => {
                let first_bytes = Self::get_first_bytes(path).unwrap();
                shortcut.insert(id, first_bytes.clone());
                match thunk.entry(first_bytes) {
                    // call `insert` on the underlying HashProxy
                    Entry::Occupied(mut occ_entry) => {
                        occ_entry.get_mut().insert(id, Duplicates::from(path))
                    }
                    // not there: create a new HashProxy
                    Entry::Vacant(vac_entry) => {
                        let hp = HashProxy::new(id, Duplicates::from(path));
                        vac_entry.insert(hp);
                    }
                }
            }
            // Add another path and its first bytes if we're a Delay
            &mut FirstKBytesProxy::Delay { .. } => self.transition(id, path),
        }
    }
}

// // // // // // // // // // // // // // // // // // // // //

pub enum HashProxy {
    Delay { id: ID, dups: Duplicates },
    Thunk {
        //thunk: HashMap<Hash, Duplicates>,
        thunk: HashMap<Hash, HashMap<ID, Duplicates>>,
        shortcut: HashMap<ID, Hash>,
    },
}

impl HashProxy {
    fn new(id: ID, dups: Duplicates) -> Self {
        HashProxy::Delay { id, dups }
    }
    // helper fn to hash a file
    fn get_hash(path: &Path) -> io::Result<Hash> {
        // not buffered for now
        let mut f = File::open(path)?;
        let mut v = vec![];
        f.read_to_end(&mut v)?;
        Ok(*md5::compute(v))
    }

    // get all repeats under this node
    fn get_repeats(&self) -> Vec<Vec<Duplicates>> {
        match self {
            &HashProxy::Delay { .. } => vec![],
            &HashProxy::Thunk { ref thunk, .. } => {
                thunk
                    .iter()
                    .filter_map(|(_hash, repeats)| {
                        if repeats.len() < 2 {
                            // only include entries that have >1 'repeat'
                            None
                        } else {
                            Some(repeats.iter().map(|(_id, dups)| dups.clone()).collect())
                        }
                    })
                    .collect()
            }
        }
    }

    fn transition(&mut self, new_id: ID, new_dups: Duplicates) {
        // convert Delay to Thunk
        let (del_id, del_dups) = match *self {
            HashProxy::Delay { id, ref mut dups } => {
                assert!(id != new_id);
                (id, dups.clone())
            }
            _ => unreachable!(),
        };
        let mut thunk = HashMap::new();
        let mut shortcut = HashMap::new();

        let new_hash = Self::get_hash(new_dups.get_path()).unwrap();
        let old_hash = Self::get_hash(del_dups.get_path()).unwrap();

        shortcut.insert(new_id, new_hash.clone());
        shortcut.insert(del_id, old_hash.clone());

        if new_hash == old_hash {
            // add both dups to the same map (or the first would be overwritten)
            let mut hm = HashMap::new();
            hm.insert(new_id, new_dups);
            hm.insert(del_id, del_dups);
            thunk.insert(old_hash, hm);
        } else {
            // add dups to different maps
            let mut new_hm = HashMap::new();
            new_hm.insert(new_id, new_dups);
            thunk.insert(new_hash, new_hm);
            let mut old_hm = HashMap::new();
            old_hm.insert(del_id, del_dups);
            thunk.insert(old_hash, old_hm);
        }

        *self = HashProxy::Thunk { thunk, shortcut };
    }

    fn insert(&mut self, id: ID, dups: Duplicates) {
        match self {
            // hard link is contained in Delay: just append it
            &mut HashProxy::Delay {
                id: id2,
                dups: ref mut dups2,
            } if id == id2 =>
            {
                dups2.append(dups);
            }
            // just add file and its hash to the thunk
            &mut HashProxy::Thunk {
                ref mut thunk,
                ref mut shortcut,
            } => {
                let hash = Self::get_hash(dups.get_path()).unwrap();
                shortcut.insert(id, hash.clone());
                match thunk.entry(hash) {
                    Entry::Occupied(mut occ_entry) => {
                        // files are completely identical
                        // add to the repeat hashtable (the val of the thunk)
                        // either create it or append to its ID's existing entry
                        let repeats = occ_entry.get_mut();
                        repeats.entry(id).or_insert(Duplicates(vec![])).append(dups);
                    }
                    Entry::Vacant(vacant_entry) => {
                        let mut hm = HashMap::new();
                        hm.insert(id, dups);
                        vacant_entry.insert(hm);
                    }
                }
            }
            // New non-link file is added from the delay stage: transition self
            &mut HashProxy::Delay { .. } => {
                self.transition(id, dups);
            }
        }
    }
}
