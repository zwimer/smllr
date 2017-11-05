
use std::fmt::{Debug, Formatter, Result};

use super::{FileCatalog};
use super::proxy::{Duplicates, FirstKBytesProxy, HashProxy};

impl Debug for Duplicates {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for i in &self.0 {
            write!(f, "{:?},  ", i)?;
        }
        Ok(())
    }
}

impl Debug for FileCatalog {
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

impl Debug for HashProxy {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
