use vfs::VFS;
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

// include unit tests
mod test;

pub trait FileActor<V: VFS> {
    fn act<'a, S: Selector<'a, V>>(vfs: &mut V, select: &S, dups: Duplicates);
}

/// Actor that prints file names but doesn't modify the filesystem
pub struct FilePrinter;

/// Actor that deletes all but the selected file
pub struct FileDeleter;

/// Actor that replaces all but the selected file with links to it
pub struct FileLinker;

impl<V: VFS> FileActor<V> for FilePrinter {
    fn act<'a, S: Selector<'a, V>>(vfs: &mut V, select: &S, dups: Duplicates) {
        let real = select.select(vfs, &dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\t`{:?}` is a duplicate", f);
        }
    }
}

impl<V: VFS> FileActor<V> for FileDeleter {
    fn act<'a, S: Selector<'a, V>>(vfs: &mut V, select: &S, dups: Duplicates) {
        let real = select.select(vfs, &dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\tDeleting `{:?}`...", f);
            vfs.rm_file(f).expect("Couldn't delete file");
        }
    }
}

impl<V: VFS> FileActor<V> for FileLinker {
    fn act<'a, S: Selector<'a, V>>(vfs: &mut V, select: &S, dups: Duplicates) {
        let real = select.select(vfs, &dups);
        info!("`{:?}` is the true file", real);
        for f in &dups.0 {
            if f == real {
                continue;
            }
            info!("\tDeleting `{:?}`...", f);
            vfs.rm_file(f).expect("Couldn't delete file");
            info!("\t\tand replacing it with a link...");
            vfs.make_link(f, real).expect("Couldn't create link");
        }
    }
}
