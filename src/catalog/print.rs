// This file contiants the implementation of Debug for common types. 
// Debug is a comon trait used to print the entire state of an object
// In the intrest of not booring you with repitition, for all functions in this file
// Debug() returns a string which details the contents of the container. 

use std::fmt::{Debug, Formatter, Result};

use super::super::ID;
use super::FileCataloger;
use super::proxy::{Duplicates, FirstKBytesProxy, HashProxy};

impl Debug for ID {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:X}:{:X}", self.dev, self.inode)
    }
}

impl Debug for Duplicates {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[")?;
        if let Some(i) = self.0.get(0) {
            write!(f, "{:?}", i)?;
        } else {
            write!(f, "~EMPTY~")?; //something's probably wrong as ATM this object only 
                                   //should be created if 2+ entries are to be added
        }
        for i in self.0.iter().skip(1) {
            write!(f, ", {:?}", i)?;
        }
        write!(f, "]")
    }
}

impl Debug for FileCataloger {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for (size, fkbp) in &self.catalog {
            writeln!(f, " {:06}b: {:?}", size, fkbp)?;
        }
        Ok(())
    }
}

impl Debug for FirstKBytesProxy {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "FKBProxy::")?;
        match self {
            &FirstKBytesProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }
            &FirstKBytesProxy::Thunk { ref thunk, .. } => {
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

impl Debug for HashProxy {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "HashProxy::")?;
        match self {
            &HashProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }
            &HashProxy::Thunk { ref thunk, .. } => {
                write!(f, "Thunk: ")?;
                for (hash, repeats) in thunk {
                    write!(
                        f,
                        "``{:02X}{:02X}..{:02X}{:02X}'':  ",
                        hash[0],
                        hash[1],
                        hash[14],
                        hash[15]
                    )?;
                    //write!(f, "!!!{}!!!", repeats.len())?;
                    for (id, dups) in repeats {
                        write!(f, "{:?}=>{:?}, ", id, dups)?;
                    }
                }
            }
        }
        Ok(())
    }
}
