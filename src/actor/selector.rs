//! Determine which of the duplicate files shouldn't be touched

use std::cmp::Ordering;
use std::path::Path;
use std::marker::PhantomData;

use vfs::{File, MetaData, VFS};
use catalog::proxy::Duplicates;

/// Interface for choosing between files
pub trait Selector<V: VFS> {
    /// Indicate that you want the max instead of the min or vice versa
    fn reverse(&mut self);
    /// Choose which of the Paths in Duplicates is the "true" (unchanged) one
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path;
    /// Helper to be called by `select`: identify the minimum
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path;
    /// Helper to be called by `select`: identify the maximum
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path;
}

/// Choose between files based on their path
pub struct PathSelect<V: VFS> {
    reverse: bool,
    vfs: PhantomData<V>, // must be generic over VFS but don't need as field
}

/// Chose between files based on their creation date
pub struct DateSelect<V: VFS> {
    reverse: bool,
    vfs: V,
}

// constructor for PathSelect
impl<V: VFS> PathSelect<V> {
    /// Construct an empty `PathSelect`
    pub fn new(_: V) -> Self {
        PathSelect {
            reverse: false,
            vfs: PhantomData,
        }
    }
}

// constructor for DateSelect
impl<V: VFS> DateSelect<V> {
    /// Construct an empty `DateSelect`
    pub fn new(v: V) -> Self {
        DateSelect {
            reverse: false,
            vfs: v,
        }
    }
}

// implement Selector for heap/trait objects
impl<V: VFS> Selector<V> for Box<Selector<V>> {
    fn reverse(&mut self) {
        (**self).reverse();
    }
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        (**self).select(dups)
    }
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        (**self).min(dups)
    }
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        (**self).max(dups)
    }
}

// implement Selector based on filepaths
impl<V: VFS> Selector<V> for PathSelect<V> {
    fn reverse(&mut self) {
        self.reverse = true;
    }
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        // select the shallowest element (the path is the shortest)
        if self.reverse {
            self.max(dups)
        } else {
            self.min(dups)
        }
    }
    // select the file closest to the root
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .min_by(|&a_path, &b_path| {
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
            })
            .unwrap() // is only None if `dups` is empty
    }
    // select the file farthest from the root
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .max_by(|&a_path, &b_path| {
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
            })
            .unwrap()
    }
}

// helper function for comparing two Files based on their date
fn date_cmp<'a, T: File>(a: &'a T, b: &'a T) -> Ordering {
    let md_a = a.get_metadata().unwrap();
    let md_b = b.get_metadata().unwrap();
    let date_a = md_a.get_creation_time().unwrap();
    let date_b = md_b.get_creation_time().unwrap();
    date_a.cmp(&date_b)
}

// implement Selector based on modification date
impl<V: VFS> Selector<V> for DateSelect<V> {
    fn reverse(&mut self) {
        self.reverse = true;
    }
    // select the file modified most recently
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .map(|path| (path, self.vfs.get_file(path).unwrap()))
            .min_by(|&(_, ref a), &(_, ref b)| date_cmp(a, b))
            .unwrap()
            .0
    }
    // select the file modified first
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .map(|path| (path, self.vfs.get_file(path).unwrap()))
            .max_by(|&(_, ref a), &(_, ref b)| date_cmp(a, b))
            .unwrap()
            .0
    }
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        // select the newest element (the SystemTime is the largest)
        if self.reverse {
            self.min(dups)
        } else {
            self.max(dups)
        }
    }
}
