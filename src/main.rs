#[macro_use] extern crate log;
extern crate env_logger;
extern crate regex;
extern crate clap;
extern crate md5;

use clap::{App, Arg};
use env_logger::LogBuilder; 
use log::LogLevelFilter;

use std::path::{Path};
use std::ffi::OsStr;

mod walker;
pub use walker::{DirWalker};

pub mod vfs;
use vfs::RealFileSystem;

mod test;

mod catalog;
use catalog::FileCatalog;

// Temporary struct: should move once we know where 
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ID {
    dev: u64,
    inode: u64
}


//const FILE_READ_BUFFER_SIZE: usize = 4096;
const FIRST_K_BYTES: usize = 32;
//const FIRST_K_BYTES: usize = 4096;

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
        .get_matches();

    let dirs: Vec<&OsStr> = matches.values_of_os("paths").unwrap().collect();
    //matches.contains("bad_paths");
    let dirs_n: Vec<&OsStr> = match matches.is_present("bad_paths") {
        true  => matches.values_of_os("bad_paths").unwrap().collect(),
        false => vec![],
    };
    let pats_n: Vec<_> = match matches.is_present("bad_regex") {
        true  => matches.values_of("bad_regex").unwrap().collect(),
        false => vec![],
    };

    // for now print all log info
    LogBuilder::new().filter(None, LogLevelFilter::max()).init().unwrap();

    // create and customize a DirWalker over the real filesystem
    let fs = RealFileSystem;
    let paths: Vec<&Path> = dirs.iter().map(Path::new).collect();
    let dw = DirWalker::new(fs, paths)
        .blacklist_folders(dirs_n)
        .blacklist_patterns(pats_n);
    let files = dw.traverse_all();
    println!("{:?}", files.len());

    // catalog all files
    let mut fc = FileCatalog::new();
    for file in &files {
        fc.insert(file);
    }

    /*
     * TODO
     *  change firstkbytes to a hash instead of just the first 32 bytes
     *      change the Debug impl probably too
     *          maybe just change them all
     *  consider consolidating FirstKBytesProxy and HashProxy somehow
     *  register duplicates up? or maybe just fetch more efficiently
     *      get ID not just a vec of duplicates?
     *  rename `Duplicates` to `Links` or something
     *  revisit `thunk` value type of HashProxy::Thunk
     *  was FileCatalog supposed to be FileCataloger?
     *
     *
     */

    /*
    let mut fc = FileCatalog::new();
    fc.insert(Path::new("/home/owen/shared/rpi4/sdd/smllr/my_tests/alphaaa"));
    //println!("alphaaa\n{:?}\n", fc);
    fc.insert(Path::new("/home/owen/shared/rpi4/sdd/smllr/my_tests/_alpha_"));
    //println!("_alpha_\n{:?}\n", fc);
    fc.insert(Path::new("/home/owen/shared/rpi4/sdd/smllr/my_tests/betaaaa"));
    //println!("betaaaa\n{:?}", fc);
    */

    // print the duplicates
    let repeats = fc.get_repeats();
    for dups in repeats {
        println!("{:?}", dups);
    }


}

