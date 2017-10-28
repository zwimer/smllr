/*#[macro_use]*/ extern crate log;
/*#[macro_use]*/ extern crate clap;

use clap::{App, Arg};

//mod walker;
//use walker::{DirWalker};

mod vfs;

//use std::path::Path;

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
        .arg(Arg::with_name("paths_n")
             .long("skip")
             .short("x")
             .help("A folder or filename to omit")
             .multiple(true)
             .takes_value(true)
             )
        // regex to skip / include
        .arg(Arg::with_name("regex_n")
             .short("o")
             .long("skip-re")
             .help("Files whose filenames match a blacklisted regex will be skipped")
             .multiple(true)
             .takes_value(true)
             )
        .arg(Arg::with_name("regex_y")
             .short("i")
             .long("only-re")
             .help("Only files whose names match a whitelisted regex will be checked")
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

    let x: Vec<_> = matches.values_of("paths").unwrap().collect();
    println!("{:?}", x);

    //let dw = DirWalker::new(vec![Path::new(".")]);
    //println!("{:?}", dw.traverse_folder(Path::new(".")));
}
