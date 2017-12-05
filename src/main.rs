extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate md5;
extern crate regex;
extern crate tiny_keccak;

// TODO make helper module for this stuff

use clap::{App, Arg};

use std::path::Path;
use std::ffi::OsStr;

pub mod helpers;
use helpers::prettify_bytes;

pub mod walker;
use walker::DirWalker;

pub mod vfs;
use vfs::RealFileSystem;

pub mod catalog;
use catalog::FileCataloger;

pub mod actor;
use actor::{FileActor, FileDeleter, FileLinker, FilePrinter};
use actor::selector::{DateSelect, PathSelect, Selector};

pub mod hash;

fn main() {
    // build arg parser
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
    // if the user supplied blacklisted paths, collect them
    let dirs_n: Vec<&OsStr> = if matches.is_present("bad_paths") {
        matches.values_of_os("bad_paths").unwrap().collect()
    } else {
        vec![]
    };
    // if the user supplied blacklisted file regexes, collect them
    let pats_n: Vec<_> = if matches.is_present("bad_regex") {
        matches.values_of("bad_regex").unwrap().collect()
    } else {
        vec![]
    };

    // print log info to stderr
    // to alter granularity, set environmental variable RUST_LOG
    // e.g. `RUST_LOG=debug ./smllr ... 2> /tmp/smllr_log`
    env_logger::init().unwrap();

    // create and customize a DirWalker over the real filesystem
    // collect all relevant files
    let fs = RealFileSystem;
    let paths: Vec<&Path> = dirs.iter().map(Path::new).collect();
    let dw = DirWalker::new(fs, &paths)
        .blacklist_folders(dirs_n)
        .blacklist_patterns(pats_n);
    let files = dw.traverse_all();
    println!("Traversing {} files...", files.len());

    // catalog all files from the DirWalker
    // duplicates are identified as files are inserted one at a time
    // TODO reduce code duplication
    let repeats = if matches.is_present("paranoid") {
        info!("Using SHA-3");
        //let mut fc = FileCataloger::new(hash::Sha3Sum, fs);
        let mut fc: FileCataloger<RealFileSystem, hash::Sha3Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }
        fc.get_repeats()
    } else {
        unimplemented!()
        /*
        info!("Using MD5");
        let mut fc = FileCataloger::new(hash::Md5Sum, fs);
        for file in &files {
            fc.insert(file);
        }
        fc.get_repeats()
        */
    };

    // use a Box to put the Selector and Actor on the heap as trait objects
    // different selectors or actors are different sizes (e.g. test_fs contains
    //  lots of data but real_fs has none), and the stack size must be known
    //  at compile time but the selector type is only known at runtime
    //  all boxes are the same size (a pointer)
    // this has the same small performance hit as C++ inheritance because it
    //  is basically a vtable
    // this works because we impl'd these traits for Box<T>

    // select which of the duplicates are "true" and act on the others
    let mut selector: Box<Selector<RealFileSystem>> = {
        // `--newest-file` or `--path-len` (default)
        if matches.is_present("newest-file") {
            Box::new(DateSelect::new(fs))
        } else {
            Box::new(PathSelect::new(fs))
        }
    };
    // invert selector if necessary (e.g. use longest path instead of shortest)
    if matches.is_present("invert-selector") {
        selector.reverse();
    }
    let selector = selector; // remove mutability

    // determine what action should be taken on non-selected files
    let mut actor: Box<FileActor<RealFileSystem, Box<Selector<RealFileSystem>>>> = {
        // `--link`, `--delete`, or `--print` (default)
        if matches.is_present("link") {
            Box::new(FileLinker::new(fs, selector))
        } else if matches.is_present("delete") {
            Box::new(FileDeleter::new(fs, selector))
        } else {
            Box::new(FilePrinter::new(fs, selector))
        }
    };

    // act on all sets of duplicates
    if repeats.is_empty() {
        println!("No duplicates found");
    } else {
        println!("Acting on {} sets of duplicates...", repeats.len());
        let mut saved_bytes = 0;
        for dups in repeats {
            saved_bytes += actor.act(dups);
        }
        println!("Idenfied {}", prettify_bytes(saved_bytes));
    }
}
