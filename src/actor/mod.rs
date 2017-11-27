use vfs::VFS;
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

use std::marker::PhantomData;

// include unit tests
mod test;

pub trait FileActor<V: VFS, S: Selector<V>> {
    //fn new(v: V, s: S) -> Self;
    fn act(&mut self, dups: Duplicates);
}

/// Actor that prints file names but doesn't modify the filesystem
pub struct FilePrinter<V: VFS, S: Selector<V>> {
    selector: S,
    //vfs: V,
    vfs: PhantomData<V>,
}

/// Actor that deletes all but the selected file
pub struct FileDeleter<V: VFS, S: Selector<V>> {
    selector: S,
    vfs: V,
}

/// Actor that replaces all but the selected file with links to it
pub struct FileLinker<V: VFS, S: Selector<V>> {
    selector: S,
    vfs: V,
}

impl<V: VFS, S: Selector<V>> FilePrinter<V, S> {
    pub fn new(_: V, s: S) -> Self {
        FilePrinter { 
            selector: s,
            vfs: PhantomData,
        }
    }
}

impl<V: VFS, S: Selector<V>> FileDeleter<V, S> {
    pub fn new(v: V, s: S) -> Self {
        FileDeleter { 
            selector: s,
            vfs: v,
        }
    }
}

impl<V: VFS, S: Selector<V>> FileLinker<V, S> {
    pub fn new(v: V, s: S) -> Self {
        FileLinker { 
            selector: s,
            vfs: v,
        }
    }
}

impl<V: VFS, S: Selector<V>> FileActor<V, S> for FilePrinter<V, S> {
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\t`{:?}` is a duplicate", f);
        }
    }
}

impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileDeleter<V, S> {
    //fn new(v: V, s: S) -> Self {
    //    FileDeleter {
    //        vfs: v,
    //        selector: s,
    //    }
    //}
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\tDeleting `{:?}`...", f);
            self.vfs.rm_file(f).expect("Couldn't delete file");
        }
    }
}

impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileLinker<V, S> {
    //fn new(v: V, s: S) -> Self {
    //    FileLinker {
    //        vfs: v,
    //        selector: s,
    //    }
    //}
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\tDeleting `{:?}`...", f);
            self.vfs.rm_file(f).expect("Couldn't delete file");
            info!("\t\tand replacing it with a link...");
            self.vfs.make_link(f, real).expect("Couldn't create link");
        }
    }
}
