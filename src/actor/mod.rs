//! Handle duplicates

use vfs::{File, MetaData, VFS};
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

mod test; // include unit tests

/// Trait for acting on duplicate files
pub trait FileActor<V: VFS, S: Selector<V>> {
    /// Use Selector `S` to identify the 'true' file and then perform its action, returning the
    /// amount of duplicate space
    fn act(&mut self, dups: Duplicates) -> u64;
}

// call FileActor methods on objects on the heap that support it
impl<V: VFS, S: Selector<V>> FileActor<V, S> for Box<FileActor<V, S>> {
    fn act(&mut self, dups: Duplicates) -> u64 {
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
    /// Create a new `FilePrinter`
    pub fn new(v: V, s: S) -> Self {
        FilePrinter {
            selector: s,
            vfs: v,
        }
    }
}

// constructors for FileDeleter: dependency inject a Selector
impl<V: VFS, S: Selector<V>> FileDeleter<V, S> {
    /// Create a new `FileDeleter`
    pub fn new(v: V, s: S) -> Self {
        FileDeleter {
            selector: s,
            vfs: v,
        }
    }
}

// constructors for FileLinker: dependency inject a Selector
impl<V: VFS, S: Selector<V>> FileLinker<V, S> {
    /// Create a new `FileLinker`
    pub fn new(v: V, s: S) -> Self {
        FileLinker {
            selector: s,
            vfs: v,
        }
    }
}

// implement `act()` for a FilePrinter
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FilePrinter<V, S> {
    /// Simply print which file in the set is considered the 'true' file and which are
    /// 'duplicates' of it as well as how much space would be saved by
    /// deleting them
    fn act(&mut self, dups: Duplicates) -> u64 {
        // identify true file with selector S
        let real = self.selector.select(&dups);
        // get the size; need to know how much space we're freeing
        let size = self.vfs
            .get_file(real)
            .expect("Failed to get file from path")
            .get_metadata()
            .expect("Failed to get file metadata")
            .get_len();
        let mut save_size = 0;
        // log the selection
        info!("{:?} is the true file", real);
        // print the file that is considered 'true'
        println!("{:?} is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            info!("\t{:?} is a duplicate", f);
            println!("\t{:?} is a duplicate", f);
            // keep track of how much space we could save (in bytes)
            save_size += size;
        }
        //log the amount of space that could be saved
        info!(
            "You can save {} bytes by deduplicating this file",
            save_size
        );
        save_size
    }
}

// implement `act()` for a FileDeleter
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileDeleter<V, S> {
    /// Print what files are duplicated and have been deleted, which one is considered
    /// the 'true', and how much space has been freed
    fn act(&mut self, dups: Duplicates) -> u64 {
        //Get the file we arn't deleteing from the selector
        let real = self.selector.select(&dups);
        let size = self.vfs
            .get_file(real)
            .expect("Failed to get file from path")
            .get_metadata()
            .expect("Failed to get file metadata")
            .get_len(); //get the size from the filesystem
        let mut save_size = 0;
        //Log which file we are not deleting
        info!("{:?} is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            // log that we will delete them
            info!("\tDeleting {:?}...", f);
            self.vfs.rm_file(f).expect("Couldn't delete file");
            // delete vfs handles logging and error printing in the case of errors
            save_size += size; //and increment the amount of space freed
        }
        //log the amount of space freed
        info!("You saved {} bytes by deduplicating this file", save_size);
        save_size
    }
}

// implement `act()` for a FileLinker
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileLinker<V, S> {
    /// Print which file is the 'true' and which have been replaced with hardlinks to
    /// the that file (and are thus effectively that file), along with
    /// how much space has been freed
    fn act(&mut self, dups: Duplicates) -> u64 {
        // Select the File:
        // get the file, metadata, size, and device from the vfs
        let real = self.selector.select(&dups);
        let real_file = self.vfs.get_file(real).expect("Couldn't find link dst");
        let real_md = real_file.get_metadata().expect("Couldn't get link dst md");
        let real_dev = real_md.get_device().expect("Couldn't get link dst device");
        let size = real_md.get_len();
        let mut save_size = 0;
        //log the 'real' file
        info!("{:?} is the true file", real);
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            // Check that we can create a hardlink
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
            // If not, inform the user.
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
                //If we can, log and print that we are deleting of the file
                info!("\tDeleting {:?}...", f);
                //println!("\tDeleting `{:?}`...", f);
                //And deleting it.
                self.vfs.rm_file(f).expect("Couldn't delete file");
                //log and print that we are replacing it with a link
                info!("\t\tand replacing it with a link to {:?}...", real);
                //println!("\t\tand replacing it with a link to `{:?}`...", real);
                //and link.
                self.vfs.make_link(f, real).expect("Couldn't create link");
                //and increment the amount of space we save
                save_size += size;
            }
        }
        // and log and print how much space was saved
        info!("You saved {} bytes by deduplicating this file", save_size);
        //println!("You saved {} bytes by deduplicating this file", save_size);
        save_size
    }
}
