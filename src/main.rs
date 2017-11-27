extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate md5;
extern crate regex;

use clap::{App, Arg};

use std::path::Path;
use std::ffi::OsStr;

mod walker;
pub use walker::DirWalker;

pub mod vfs;
use vfs::RealFileSystem;

mod catalog;
use catalog::FileCataloger;

mod actor;
pub use actor::{FileActor, FileDeleter, FileLinker, FilePrinter};
use actor::selector::{DateSelect, PathSelect, Selector};

// Helpers:

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ID {
    dev: u64,
    inode: u64,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FirstBytes(pub(crate) [u8; FIRST_K_BYTES]);

pub type Hash = [u8; 16];

const FIRST_K_BYTES: usize = 32;


fn main() {
    let matches = App::new("smllr")
        // paths without an argument after
        .arg(Arg::with_name("paths")
             .help("List of files or directories to deduplicate")
             .multiple(true)
             .takes_value(true)
             .required(true)
             )
        // paths to skip (`--skip /tmp --skip /usr`)
        .arg(Arg::with_name("bad_paths")
             .long("skip")
             .short("x")
             .help("A folder or filename to omit")
             .multiple(true)
             .takes_value(true)
             )
        // regex to skip / include
        .arg(Arg::with_name("bad_regex")
             .short("o")
             .long("skip-re")
             .help("Files whose filenames match a blacklisted regex will be skipped")
             .multiple(true)
             .takes_value(true)
             )
        // paranoid flag
        .arg(Arg::with_name("paranoid")
             .short("p")
             .long("paranoid")
             .help("Use SHA-3 to hash files instead of MD5")
             )
        // determine selector
        .arg(Arg::with_name("path-len")
             .long("path-len")
             .conflicts_with("newest-file")
             .help("Preserve the file closest to the root (default)")
             )
        .arg(Arg::with_name("newest-file")
             .long("newest-file")
             .help("Preserve the file that was made most recently")
             )
        .arg(Arg::with_name("invert-selector")
             .long("invert-selector")
             .help("Invert the selector criterion (e.g. preserve the deepest file)")
             )
        // determine actor
        .arg(Arg::with_name("print")
             .long("print")
             .conflicts_with("delete")
             .conflicts_with("link")
             .help("Print duplicate files (default)")
             )
        .arg(Arg::with_name("delete")
             .conflicts_with("link")
             .long("delete")
             .help("Delete duplicate files")
             )
        .arg(Arg::with_name("link")
             .long("link")
             .help("Replace duplicate files with hard links")
             )
        .get_matches();

    // decide which files are fair game
    let dirs: Vec<&OsStr> = matches.values_of_os("paths").unwrap().collect();
    let dirs_n: Vec<&OsStr> = if matches.is_present("bad_paths") {
        matches.values_of_os("bad_paths").unwrap().collect()
    } else {
        vec![]
    };
    let pats_n: Vec<_> = if matches.is_present("bad_regex") {
        matches.values_of("bad_regex").unwrap().collect()
    } else {
        vec![]
    };

    // print all log info
    /*
    env_logger::LogBuilder::new()
        .filter(None, log::LogLevelFilter::max())
        .init()
        .unwrap();
        */

    // create and customize a DirWalker over the real filesystem
    let fs = RealFileSystem;
    let paths: Vec<&Path> = dirs.iter().map(Path::new).collect();
    let dw = DirWalker::new(fs, &paths)
        .blacklist_folders(dirs_n)
        .blacklist_patterns(pats_n);
    let files = dw.traverse_all();

    // catalog all files
    let mut fc = FileCataloger::new(fs);
    for file in &files {
        fc.insert(file);
    }

    // identify repeats
    let repeats = fc.get_repeats();

    // select and act on them
    let mut selector: Box<Selector<RealFileSystem>> = {
        if matches.is_present("newest-file") {
            Box::new(DateSelect::new(fs))
        } else {
            Box::new(PathSelect::new(fs))
        }
    };
    if matches.is_present("invert-selector") {
        selector.reverse();
    }
    let selector = selector; // remove mutability

    let mut actor: Box<FileActor<RealFileSystem, Box<Selector<RealFileSystem>>>> = {
        if matches.is_present("link") {
            Box::new(FileLinker::new(fs, selector))
        } else if matches.is_present("delete") {
            Box::new(FileDeleter::new(fs, selector))
        } else {
            Box::new(FilePrinter::new(fs, selector))
        }
    };

    /*
     * TODO
     *  change firstkbytes to a hash instead of just the first 32 bytes
     *      change the Debug impl probably too
     *          maybe just change them all
     *  consider consolidating FirstKBytesProxy and HashProxy somehow
     *  register duplicates up? or maybe just fetch more efficiently
     *      get ID not just a vec of duplicates?
     */

    // print the duplicates

    for dups in repeats {
        actor.act(dups);
    }
}
