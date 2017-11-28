use vfs::{File, MetaData, VFS};
use catalog::proxy::Duplicates;

pub mod selector;
use self::selector::Selector;

mod test; // include unit tests

/// Trait for acting on duplicate files
pub trait FileActor<V: VFS, S: Selector<V>> {
    /// FileActor<V, S>.act(Duplicates) lets selector S select the file 
     /// in duplicates which is considered the 'true' file and then 
     /// 'acts' in a manner apropriate to the fileactor.
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
     /// <FilePrinter<V, S> as FileActor<V, S> >.act(), we simply print 
     /// which file in the set is considered the 'true' file and which are 
     /// 'duplicates' of it as well as how much space would be saved by 
	  /// deleting them.
     fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups); // identify true file with selector S
        let size = self.vfs
            .get_file(real)
            .unwrap()
            .get_metadata()
            .unwrap()
            .get_len();// Get The size from the filesystem
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);//log the selection
		  println!("`{:?}` is the true file", real);//print the file that is considered 'true'
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            info!("\t`{:?}` is a duplicate", f);// Log as duplicate
            println!("\t`{:?}` is a duplicate", f);// and inform the user
            save_size += size;//increment the amount of space we could save by size
        }
		  //log the amount of space that could be saved
        info!(
            "You can save {} bytes by deduplicating this file",
            save_size
        );
		  //print the amount of space that could be saved to the user
        println!(
            "You can save {} bytes by deduplicating this file",
            save_size
        );
    }
}

// implement `act()` for a FileDeleter
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileDeleter<V, S> {
    /// <FileDeleter<V, S> as FileActor<V, S> >.act(), we simply print what 
    /// files are duplicated and have been deleted, which one is considered 
	 /// the 'true' and which are and how much space has been freed. 
    fn act(&mut self, dups: Duplicates) {
	     //Get the file we arn't deleteing from the selector
        let real = self.selector.select(&dups);
        let size = self.vfs
            .get_file(real)
            .unwrap()
            .get_metadata()
            .unwrap()
            .get_len();//get the size from the filesystem
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);//Log which file we are not deleting
        println!("`{:?}` is the true file", real);//and inform the user
        // iterate over all other duplicates
        for f in dups.0.iter().filter(|&f| f.as_path() != real) {
            info!("\tDeleting `{:?}`...", f);// log that we will delete them
            println!("\tDeleting `{:?}`...", f);// and inform the user
            self.vfs.rm_file(f).expect("Couldn't delete file");// delete
				//vfs handles logging and error printing in the case of errors
            save_size += size;//and increment the amount of space freed
        }
        //log the amount of space freed
        info!("You saved {} bytes by deduplicating this file", save_size);
		  println!("You saved {} bytes by deduplicating this file", save_size);// and inform the user
    }
}

// implement `act()` for a FileLinker
impl<V: VFS, S: Selector<V>> FileActor<V, S> for FileLinker<V, S> {
    /// <FileLinker<V, S> as FileActor<V, S> >.act(), we print which file
	 /// is the 'original' and which have been replaced with hardlinks to
	 /// the that file (and are thus effectively that file), along with  
	 /// how much space has been freed. 
    fn act(&mut self, dups: Duplicates) {
        let real = self.selector.select(&dups);//Select the File
		  // get the file, metadata, size, and device from the vfs
        let real_file = self.vfs.get_file(real).expect("Couldn't find link dst");
        let real_md = real_file.get_metadata().expect("Couldn't get link dst md");
        let real_dev = real_md.get_device().expect("Couldn't get link dst device");
        let size = real_md.get_len();
        let mut save_size = 0;
        info!("`{:?}` is the true file", real);//log the 'real' file
        println!("`{:?}` is the true file", real);//and inform the user
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
                info!("\tDeleting `{:?}`...", f);
                println!("\tDeleting `{:?}`...", f);
					 //And deleting it.
                self.vfs.rm_file(f).expect("Couldn't delete file");
					 //log and print that we are replacing it with a link
                info!("\t\tand replacing it with a link to `{:?}`...", real);
                println!("\t\tand replacing it with a link to `{:?}`...", real);
					 //and link.
                self.vfs.make_link(f, real).expect("Couldn't create link");
					 //and increment the amount of space we save
                save_size += size;
            }
        }
		  // and log and print how much space was saved
        info!("You saved {} bytes by deduplicating this file", save_size);
        println!("You saved {} bytes by deduplicating this file", save_size);
    }
}
