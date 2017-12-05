use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::collections::hash_map::Entry;

use vfs::{File, VFS};
use super::ID;
use helpers::FirstBytes;
use hash::FileHash;

// Duplicates is a decorator for a vector of pathbufs which represents
// a set of files. In code, it is an invariant that any 2 files in a
// duplicates are identicle.
#[derive(Clone)]
/// Collection of `PathBuf`s that point to identical files
pub struct Duplicates(pub(crate) Vec<PathBuf>);

impl Duplicates {
    // Convert a path to a vector of length 1 containing that path
    fn from(path: &Path) -> Self {
        Duplicates(vec![path.to_path_buf()])
    }
    // Convert the first element to a path
    fn get_path(&self) -> &Path {
        &self.0[0]
    }
    // Add path to this Duplicates
    fn push(&mut self, path: &Path) {
        self.0.push(path.to_path_buf());
    }
    // Add all elements in another object othr to this Duplicates
    fn append(&mut self, mut othr: Duplicates) {
        self.0.append(&mut othr.0);
    }
}

// Begin FirstKBytesProxy

/// Proxy of firstbytes: until two elements have been added, there is no
/// chance of a collision so put off constructing the hashmap and shortcut
pub enum FirstKBytesProxy<H: FileHash> {
    // in the first state there is one file
    // don't look up its first k bytes unless it has the same size as another
    Delay {
        id: ID,
        dups: Duplicates,
    },
    // after 2 files with the first k bytes have been found, store them
    // also maintain a shortcut for looking up their values by their id
    // for hardlink detection.
    Thunk {
        thunk: HashMap<FirstBytes, HashProxy<H>>,
        shortcut: HashMap<ID, FirstBytes>,
    },
}


impl<H: FileHash> FirstKBytesProxy<H> {
    /// Construct a new FirstKBytesProxy with delay of path
    pub fn new(id: ID, path: &Path) -> Self {
        FirstKBytesProxy::Delay {
            id,
            dups: Duplicates::from(path),
        }
    }

    /// Traverse contained `HashProxy`s and identify contents with more than one
    /// path associated with it
    pub(super) fn get_repeats(&self) -> Vec<Duplicates> {
        match *self {
            // in the Delay state, return `dups` if it contains multiple paths
            FirstKBytesProxy::Delay { ref dups, .. } => if dups.0.len() >= 2 {
                vec![dups.clone()]
            } else {
                vec![]
            },
            // in the Thunk state, traverse all `HashProx`s
            FirstKBytesProxy::Thunk { ref thunk, .. } => {
                thunk.iter().fold(vec![], |mut acc, (_fb, hp)| {
                    acc.append(&mut hp.get_repeats());
                    acc
                })
            }
        }
    }

    /// Transition type from a Delay to a Thunk with the introduction of a new file
    /// Preview both files and add them to the contents of the new Thunk
    fn transition<T: VFS>(&mut self, vfs: &T, new_id: ID, new_path: &Path) {
        // convert from a Delay to a Thunk
        // panics if new belongs in Delay.dups
        // panics if `self` is of type Thunk
        // NOTE this involves EITHER a clone of dups OR a promise-violating hack
        let (del_id, del_dups) = match *self {
            FirstKBytesProxy::Delay { id, ref mut dups } => {
                // this a somewhat hacky potential future speedup
                // if there are problems with Duplicates being empty, look here
                // "steal" `dups` so we don't have to clone it
                // but we can't just take it because we can't consume self
                // OPTION A: the possibly dangerous but more efficient one:
                //let stolen_dups = ::std::mem::replace(dups, Duplicates(vec![]));
                //(id, stolen_dups)
                // OPTION B: the safer but more expensive version:
                (id, dups.clone())
            }
            _ => unreachable!(),
        };
        assert!(new_id != del_id);
        // Initialize new type's variables
        let mut thunk = HashMap::new();
        let mut shortcut = HashMap::new();

        // get first bytes of both files
        let new_file = vfs.get_file(new_path).unwrap();
        let old_file = vfs.get_file(&del_dups.0[0]).unwrap();
        let new_first_bytes = new_file.get_first_bytes().unwrap();
        let old_first_bytes = old_file.get_first_bytes().unwrap();

        // and add them to the map's shortcut.
        shortcut.insert(new_id, new_first_bytes.clone());
        shortcut.insert(del_id, old_first_bytes.clone());
        // construct duplicate wraper around the new path and insert into new
        // hashmap.
        let new_dups = Duplicates::from(new_path);
        if new_first_bytes == old_first_bytes {
            let mut hp = HashProxy::new(del_id, del_dups);
            hp.insert(vfs, new_id, new_dups);
            thunk.insert(old_first_bytes, hp);
        } else {
            thunk.insert(new_first_bytes, HashProxy::new(new_id, new_dups));
            thunk.insert(old_first_bytes, HashProxy::new(del_id, del_dups));
        }
        // replace pointer from delay a pointer to thunk.
        *self = FirstKBytesProxy::Thunk { thunk, shortcut };
    }

    /// Add a new path to the proxy
    pub fn insert<T: VFS>(&mut self, vfs: &T, id: ID, path: &Path) {
        match *self {
            // If a hard link and self is a Delay, insert a hard link to what's
            // already stored in Delay
            FirstKBytesProxy::Delay {
                id: id2,
                ref mut dups,
                ..
            } if id == id2 =>
            {
                dups.push(path);
            }
            // If self is a thunk get first bytes and add to shortcut.
            // If a match for a proxy, add
            // to the proxy; otherwise create a new hashproxy.
            FirstKBytesProxy::Thunk {
                ref mut thunk,
                ref mut shortcut,
            } => {
                let file = vfs.get_file(path).unwrap();
                let first_bytes = file.get_first_bytes().unwrap();
                //let first_bytes = Self::get_first_bytes(path).unwrap();
                shortcut.insert(id, first_bytes.clone());
                match thunk.entry(first_bytes) {
                    // call `insert` on the underlying HashProxy
                    Entry::Occupied(mut occ_entry) => {
                        occ_entry.get_mut().insert(vfs, id, Duplicates::from(path))
                    }
                    // not there: create a new HashProxy
                    Entry::Vacant(vac_entry) => {
                        let hp = HashProxy::new(id, Duplicates::from(path));
                        vac_entry.insert(hp);
                    }
                }
            }
            // If we are a delay and need to insert a path that is not a hardlink,
            // transition to a thunk
            FirstKBytesProxy::Delay { .. } => self.transition(vfs, id, path),
        }
    }
}

// Begin HashProxy

/// Proxy of hashes: until two elements have been added, there is no
/// chance of a collision so put off constructing the hashmap and shortcut
pub enum HashProxy<H: FileHash> {
    // only one unique element has been added
    Delay {
        id: ID,
        dups: Duplicates,
    },
    // need to map `Hash`es to a set of `Duplicates`
    Thunk {
        thunk: HashMap<<H as FileHash>::Output, Duplicates>,
        shortcut: HashMap<ID, <H as FileHash>::Output>,
    },
    // see `FirstKBytesProxy` for more documentation
    // major difference is that `Duplicates` can contain non-hardlinks
}


// closely parallels FirstKBytesProxy's documentation
impl<H: FileHash> HashProxy<H> {
    //Construct a new hashprxy. As only 1 object, will be of the Delay type.
    fn new(id: ID, dups: Duplicates) -> Self {
        HashProxy::Delay { id, dups }
    }

    // get all repeats under this node and return as a set of sets of duplicates.
    /// Check all Duplicates for files associated with multiple Paths
    fn get_repeats(&self) -> Vec<Duplicates> {
        match *self {
            HashProxy::Delay { ref dups, .. } => if dups.0.len() >= 2 {
                vec![dups.clone()]
            } else {
                vec![]
            },
            HashProxy::Thunk { ref thunk, .. } => {
                thunk
                    .iter()
                    .filter_map(|(_hash, repeats)| {
                        if repeats.0.len() >= 2 {
                            // if there are 2 or more elements
                            // (including 2 links to 1 file)
                            Some(repeats.clone())
                        } else {
                            // exactly one representation on the hard drive
                            None
                        }
                    })
                    .collect()
            }
        }
    }
    // private helper fuction which handles the conversion from Delay to HashProxy::Thunk
    fn transition<T: VFS>(&mut self, vfs: &T, new_id: ID, new_dups: Duplicates) {
        // convert Delay to Thunk
        let (del_id, del_dups) = match *self {
            HashProxy::Delay { id, ref mut dups } => {
                assert!(id != new_id);
                (id, dups.clone())
            }
            _ => unreachable!(),
        };
        //Set up variables for thunk state
        let mut thunk: HashMap<<H as FileHash>::Output, Duplicates> = HashMap::new();
        let mut shortcut = HashMap::new();

        // get hashes
        let new_file = vfs.get_file(new_dups.get_path()).unwrap();
        let old_file = vfs.get_file(del_dups.get_path()).unwrap();
        let new_hash: <H as FileHash>::Output = new_file.get_hash::<H>().unwrap();
        let old_hash: <H as FileHash>::Output = old_file.get_hash::<H>().unwrap();

        // insert into shortcut
        shortcut.insert(new_id, new_hash.clone());
        shortcut.insert(del_id, old_hash.clone());

        // thunk: HashMap < Hash, Duplicates >
        thunk.insert(new_hash, new_dups);
        thunk
            .entry(old_hash)
            .or_insert_with(|| Duplicates(vec![]))
            .append(del_dups);

        // set our pointer to the new thunk state.
        *self = HashProxy::Thunk { thunk, shortcut };
    }

    // insert Duplicate into the data structure
    fn insert<T: VFS>(&mut self, vfs: &T, id: ID, dups: Duplicates) {
        match *self {
            // if its just a hard link and we are in Delay: just append it
            HashProxy::Delay {
                id: id2,
                dups: ref mut dups2,
                //..
            } if id == id2 =>
            {
                dups2.append(dups);
            }
            // If we are in a thunk state, just add file and its hash
            HashProxy::Thunk {
                ref mut thunk,
                ref mut shortcut,
            } => {
                let file = vfs.get_file(dups.get_path()).unwrap();
                let hash: <H as FileHash>::Output = file.get_hash::<H>().unwrap();
                shortcut.insert(id, hash.clone());
                match thunk.entry(hash) {
                    Entry::Occupied(mut occ_entry) => {
                        // if files are completely identical
                        // add to the repeat hashtable (the val of the thunk)
                        // either create it or append to its ID's existing entry
                        let repeats = occ_entry.get_mut();
                        repeats.append(dups);
                    } //  Otherwise just add it to a new hashmap.
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(dups);
                    }
                }
            }
            // if a new non-link file is added while self is a delay stage: transition to Thunk
            HashProxy::Delay { .. } => {
                self.transition(vfs, id, dups);
            }
        }
    }
}
