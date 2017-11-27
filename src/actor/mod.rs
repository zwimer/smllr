use vfs::{File, MetaData, VFS};
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

mod test; // include unit tests

/// Trait for acting on duplicate files
pub trait FileActor<V: VFS, S: Selector<V>> {
    fn act(&mut self, dups: Duplicates);
}

// call FileActor methods on objects on the heap that support it
impl<V: VFS, S: Selector<V>> FileActor<V, S> for Box<FileActor<V, S>> {
    fn act(&mut self, dups: Duplicates) {
        (**self).act(dups)
    }
}

/// Actor that prints file names but doesn't modify the filesystem
pub struct FilePrinter<V: VFS, S: Selector<V>> {
    selector: S,
    vfs: V,
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

// constructors for FilePrinter: dependency inject a Selector
impl<V: VFS, S: Selector<V>> FilePrinter<V, S> {
    pub fn new(v: V, s: S) -> Self {
        FilePrinter {
            selector: s,
            vfs: v,
        }
    }
}

// constructors for FileDeleter: dependency inject a Selector
impl<V: VFS, S: Selector<V>> FileDeleter<V, S> {
    pub fn new(v: V, s: S) -> Self {
        FileDeleter {
            selector: s,
            vfs: v,
        }
    }
}

// constructors for FileLinker: dependency inject a Selector
impl<V: VFS, S: Selector<V>> FileLinker<V, S> {
    pub fn new(v: V, s: S) -> Self {
        FileLinker {
            selector: s,
            vfs: v,
        }
    }
}

// implement `act()` for a FilePrinter
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FilePrinter<V, S> {
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups); // identify true file
        let size = self.vfs
            .get_file(real)
            .unwrap()
            .get_metadata()
            .unwrap()
            .get_len();
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);
        println!("`{:?}` is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            info!("\t`{:?}` is a duplicate", f);
            println!("\t`{:?}` is a duplicate", f);
            save_size += size;
        }
        info!(
            "You can save {} bytes by deduplicating this file",
            save_size
        );
        println!(
            "You can save {} bytes by deduplicating this file",
            save_size
        );
    }
}

// implement `act()` for a FileDeleter
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileDeleter<V, S> {
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);
        let size = self.vfs
            .get_file(real)
            .unwrap()
            .get_metadata()
            .unwrap()
            .get_len();
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);
        println!("`{:?}` is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            info!("\tDeleting `{:?}`...", f);
            println!("\tDeleting `{:?}`...", f);
            self.vfs.rm_file(f).expect("Couldn't delete file");
            save_size += size;
        }
        info!("You saved {} bytes by deduplicating this file", save_size);
        println!("You saved {} bytes by deduplicating this file", save_size);
    }
}

// implement `act()` for a FileLinker
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileLinker<V, S> {
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);
        let real_file = self.vfs.get_file(real).expect("Couldn't find link dst");
        let real_md = real_file.get_metadata().expect("Couldn't get link dst md");
        let real_dev = real_md.get_device().expect("Couldn't get link dst device");
        let size = real_md.get_len();
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);
        println!("`{:?}` is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            let f_dir = f.parent().unwrap(); // can't be a dir so can't be "/"
            let f_dir_file = self.vfs
                .get_file(f_dir)
                .expect("Couldn't find link src parent");
            let f_dir_md = f_dir_file
                .get_metadata()
                .expect("Couldn't get link src parent md");
            let f_dir_dev = f_dir_md
                .get_device()
                .expect("Couldn't get link src parent device");

            if real_dev != f_dir_dev {
                warn!(
                    "You tried to create a link from directory `{:?}` on device {:?} \
                     to the file `{:?}` on device {:?}.\n\
                     Hard-linking across devices is generally an error. \
                     Skipping...",
                    f_dir,
                    f_dir_dev,
                    real,
                    real_dev
                );
            } else {
                info!("\tDeleting `{:?}`...", f);
                println!("\tDeleting `{:?}`...", f);
                self.vfs.rm_file(f).expect("Couldn't delete file");
                info!("\t\tand replacing it with a link to `{:?}`...", real);
                println!("\t\tand replacing it with a link to `{:?}`...", real);
                self.vfs.make_link(f, real).expect("Couldn't create link");
                save_size += size;
            }
        }
        info!("You saved {} bytes by deduplicating this file", save_size);
        println!("You saved {} bytes by deduplicating this file", save_size);
    }
}
