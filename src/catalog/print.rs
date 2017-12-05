// This file contains the implementation of Debug for common types.
// Debug is a comon trait used to print the entire state of an object
// In the intrest of not boring you with repitition, for all functions in this file
// Debug() returns a string which details the contents of the container.
// NOTE: these are for testing purposes, the user won't see this

use std::fmt::{Debug, Formatter, Result};

use helpers::ID;
use catalog::FileCataloger;
use hash::FileHash;

use vfs::VFS;
use catalog::proxy::{Duplicates, FirstKBytesProxy, HashProxy};

// print debug info for ID
impl Debug for ID {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:X}:{:X}", self.dev, self.inode)
    }
}

// print debug info for Duplicates
impl Debug for Duplicates {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[")?;
        if let Some(i) = self.0.get(0) {
            write!(f, "{:?}", i)?;
        } else {
            // something's probably wrong as ATM this object only
            // should be created if 2+ entries are to be added
            write!(f, "~EMPTY~")?;
        }
        for i in self.0.iter().skip(1) {
            write!(f, ", {:?}", i)?;
        }
        write!(f, "]")
    }
}

// print debug info for FileCataloger
impl<T: VFS, H: FileHash> Debug for FileCataloger<T, H> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for (size, fkbp) in &self.catalog {
            writeln!(f, " {:06}b: {:?}", size, fkbp)?;
        }
        Ok(())
    }
}

// print debug info for FirstKBytesProxy
impl<H: FileHash> Debug for FirstKBytesProxy<H> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "FKBProxy::")?;
        match *self {
            FirstKBytesProxy::Delay {
                ref id, ref dups, ..
            } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }
            FirstKBytesProxy::Thunk { ref thunk, .. } => {
                write!(f, "Thunk: ")?;
                for (bytes, hp) in thunk {
                    let s = String::from_utf8_lossy(&bytes.0);
                    write!(f, "``")?;
                    for c in s.chars().take(3) {
                        write!(f, "{}", c)?;
                    }
                    write!(f, "..")?;
                    for c in s.chars().skip(29) {
                        write!(f, "{}", c)?;
                    }
                    write!(f, "'':  {:?}", hp)?;
                }
            }
        }
        Ok(())
    }
}

// print debug info for HashProxy
impl<T: FileHash> Debug for HashProxy<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "HashProxy::")?;
        match *self {
            HashProxy::Delay {
                ref id, ref dups, ..
            } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }
            HashProxy::Thunk { ref thunk, .. } => {
                write!(f, "Thunk: {:?}", thunk)?;
                //write!(f, "Thunk: ")?;
                //write!(f, "TODO HASH")?;
                /*
                for (hash, repeats) in thunk {
                    write!(
                        f,
                        "``{:02X}{:02X}..{:02X}{:02X}'':  ",
                        hash[0],
                        hash[1],
                        hash[14],
                        hash[15]
                    )?;
                    write!(f, "{:?}, ", repeats)?;
                }
                */            }
        }
        Ok(())
    }
}
