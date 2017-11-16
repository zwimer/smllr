// This file contiants the implementation of Debug for common types. 
// Debug is a comon trait used to print the entire state of an object
// In the intrest of not booring you with repitition, for all functions in this file
// Debug() returns a string which details the contents of the container. 

//RUST NOTE: the ? 'operator' is if 'OK' : .unwrap() , else if 'ERROR' : return from function.
//.unwrap() on datastructes is somtimes equivlent to .Debug, as in our case

use std::fmt::{Debug, Formatter, Result};

use super::super::ID;
use super::FileCataloger;
use super::proxy::{Duplicates, FirstKBytesProxy, HashProxy};
// print (device, inode). Note that fileid == inode
impl Debug for ID {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:X}:{:X}", self.dev, self.inode)
    }
}
// prints the list of paths in duplicates
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

// print contents of FileCataloger.
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
				//if Delay, write all id's and paths in the delay, 
            &FirstKBytesProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }//else (is thunk), write all abriviated key-value pairs
            &FirstKBytesProxy::Thunk { ref thunk, .. } => {
                //(first k-bytes in ddd..ddd form) 
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
                    }// and value in standard output for the hashproxies 
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
        match self {//if delay print id and list of duplicates
            &HashProxy::Delay { ref id, ref dups } => {
                write!(f, "Delay: ({:?})  {:?}", id, dups)?;
            }// if thunk, print hashmap in key-value pairs with key as dd..dd 
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
                    // and value as a list of paths
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
