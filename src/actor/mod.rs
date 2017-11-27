use vfs::VFS;
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

use std::marker::PhantomData;

// include unit tests
mod test;

pub trait FileActor<'a, V: VFS, S: Selector<'a, V>> {
    fn new(v: &'a V, s: S) -> Self;
    fn act(&mut self, dups: Duplicates);
}

/// Actor that prints file names but doesn't modify the filesystem
pub struct FilePrinter<'a, V: VFS + 'a, S: Selector<'a, V>> {
    selector: S,
    //vfs: V,
    vfs: PhantomData<&'a V>,
}

/// Actor that deletes all but the selected file
pub struct FileDeleter<'a, V: VFS + 'a, S: Selector<'a, V>> {
    selector: S,
    vfs: &'a V,
}

/// Actor that replaces all but the selected file with links to it
pub struct FileLinker<'a, V: VFS + 'a, S: Selector<'a, V>> {
    selector: S,
    vfs: &'a V,
}

impl<'a, V: VFS, S: Selector<'a, V>> FileActor<'a, V, S> for FilePrinter<'a, V, S> {
    fn new(_: &'a V, s: S) -> Self {
        FilePrinter { 
            selector: s,
            vfs: PhantomData,
        }
    }
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

/*
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileDeleter<V, S> {
    fn new(v: V, s: S) -> Self {
        FileDeleter {
            vfs: v,
            selector: s,
        }
    }
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
    fn new(v: V, s: S) -> Self {
        FileLinker {
            vfs: v,
            selector: s,
        }
    }
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
*/
