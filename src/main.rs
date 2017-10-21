#[macro_use] extern crate log;

mod walker;
use walker::{DirWalker};

use std::path::Path;

fn main() {
    let dw = DirWalker::new(vec![Path::new(".")]);
    println!("{:?}", dw.traverse_folder(Path::new(".")));
}
