
use std::cmp::Ordering;
use std::path::Path;

use super::vfs::{VFS, File, MetaData};
use catalog::proxy::Duplicates;

/// Interface for choosing between files
pub trait Selector<'a, V: VFS> {
    // indicate that you want the max instead of the min or vice versa
    fn reverse(self) -> Self;
    // ctor
    fn new(_: V) -> Self;
    // choose which of the Paths in Duplicates is the "true" (unchanged) one
    fn select<'b>(&self, vfs: &V, dups: &'b Duplicates) -> &'b Path;
    // helpers to be called by select
    fn min<'b>(vfs: &V, dups: &'b Duplicates) -> &'b Path;
    fn max<'b>(vfs: &V, dups: &'b Duplicates) -> &'b Path;
}

/// Choose between files based on their path
pub struct PathSelect { reverse: bool }

/// Chose between files based on their creation date
pub struct DateSelect { reverse: bool }

impl<'a, V: VFS> Selector<'a, V> for PathSelect {
    fn new(_: V) -> Self {
        PathSelect { reverse: false }
    }
    fn reverse(self) -> Self {
        PathSelect { reverse: true }
    }
    fn select<'b>(&self, vfs: &V, dups: &'b Duplicates) -> &'b Path {
        // select the shallowest element (the path is the shortest)
        if self.reverse {
            Self::max(vfs, dups)
        } else {
            Self::min(vfs, dups)
        }
    }
    fn min<'b>(_: &V, dups: &'b Duplicates) -> &'b Path {
        dups.0.iter()
            .min_by(|&a_path, &b_path| { 
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
        }).unwrap()
    }
    fn max<'b>(_: &V, dups: &'b Duplicates) -> &'b Path {
        dups.0.iter()
            .max_by(|&a_path, &b_path| { 
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
        }).unwrap()
    }
}

fn cmp<'a, T: File>(a: &'a T, b: &'a T) -> Ordering {
    let md_a = a.get_metadata().unwrap();
    let md_b = b.get_metadata().unwrap();
    let date_a = md_a.get_creation_time().unwrap();
    let date_b = md_b.get_creation_time().unwrap();
    date_a.cmp(&date_b)
}

impl<'a, V: VFS> Selector<'a, V> for DateSelect {
    fn new(_: V) -> Self {
        DateSelect { reverse: false }
    }
    fn reverse(self) -> Self {
        DateSelect { reverse: true }
    }
    fn min<'b>(vfs: &V, dups: &'b Duplicates) -> &'b Path {
        dups.0.iter()
            .map(|path| (path,vfs.get_file(path).unwrap()))
            .min_by(|&(_, ref a), &(_, ref b)| { 
                cmp(a, b)
        }).unwrap().0
    }
    fn max<'b>(vfs: &V, dups: &'b Duplicates) -> &'b Path {
        dups.0.iter()
            .map(|path| (path,vfs.get_file(path).unwrap()))
            .max_by(|&(_, ref a), &(_, ref b)| { 
                cmp(a, b)
        }).unwrap().0
    }
    fn select<'b>(&self, vfs: &V, dups: &'b Duplicates) -> &'b Path {
        // select the newest element (the SystemTime is the largest)
        if self.reverse {
            Self::min(vfs, dups)
        } else {
            Self::max(vfs, dups)
        }
    }
}


