use std::fs::File;
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::{self, Read};

extern crate md5;
extern crate walkdir;

const FOLLOW_SYMLINKS: bool = false;
const ROOT_DIRECTORY: &'static str = "/";
//const ROOT_DIRECTORY: &'static str = "/home/owen";
//const REPORT_ERRORS: bool = false;
const BUFFER_SIZE: usize = 8096;   // 20MB  // TODO: tweak

type FileIter = Box<Iterator<Item=walkdir::DirEntry>>;

/*
enum SelectionStrat {
    PathLen,
    CreationTime,
    UpdateTime,
    //Drive(String),
    Prefix(String),
    NotPrefix(String),
}
type SelectionStrats = Vec<SelectionStrat>;

enum ResolutionStrat {
    Nothing,
    Delete,
    Symlink,
}
*/

fn get_info(path: &Path) -> io::Result<(u64,[u8;16])> {
    let mut f = File::open(path)?;
    let mut buffer: Box<[u8]> = vec![0u8; BUFFER_SIZE].into_boxed_slice();
    let mut total = 0usize;
    let mut context = md5::Context::new();
    loop {
        let size = f.read(&mut buffer)?;
        if size == 0 {
            let digest = context.compute();
            return Ok((total as u64, *digest));
        }
        total += size;
        context.consume(&buffer[..size]);
        for b in buffer.iter_mut() { *b = 0; }  // maybe use mem::zeroed?
    }
}

fn file_iterator() -> FileIter {
    Box::new(
        walkdir::WalkDir::new(ROOT_DIRECTORY)
            .follow_links(FOLLOW_SYMLINKS)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let t = e.file_type();
                t.is_file() || (FOLLOW_SYMLINKS && t.is_symlink())
            })
        )
}

fn read_all(files: FileIter, total: usize) -> HashMap<[u8;16],Vec<PathBuf>> {
    // iterate through files
    let mut file_hashes: HashMap<[u8;16],(u64,PathBuf)> = HashMap::new();
    let mut duplicates: HashMap<[u8;16],Vec<PathBuf>> = HashMap::new();
    let mut successes = 0u32;
    let mut failures = 0u32;
    let mut redundant_bytes = 0u64;
    let mut total_bytes = 0u64;
    for entry in files {
        //if (successes + failures) / total 
        //println!("{}, {}, {:?}", duplicates.len(), file_hashes.len(), entry);
        if let Ok((len,hash)) = get_info(entry.path()) {
            let pathbuf = PathBuf::from(entry.path());
            let file_entry = file_hashes.entry(hash);
            total_bytes += len;
            match file_entry {
                // if we've never seen this hash, add it to the hash map (get it?)
                // if we've seen it exactly once, add both it's entry and this to dupe map
                // if it's already in the dupe map, add this to its entry
                Entry::Occupied(e) => {
                    assert_eq!(len, e.get().0, "lol md5 collision");
                    let ref first_dupe = e.get().1;
                    let mut dupe_entry = duplicates.entry(hash)
                        .or_insert(vec![first_dupe.clone()]);
                    dupe_entry.push(pathbuf);
                    redundant_bytes += len;
                },
                Entry::Vacant(_) => {
                    file_entry.or_insert((len,pathbuf));
                },
            }
            successes += 1;
        } else {
            failures += 1;
        }
    }

    println!("Scanned {} bytes", total_bytes);
    println!("Redundant bytes: {}", redundant_bytes);
    println!();
    println!("Unique files: {}", file_hashes.len());
    println!("Files with ≥1 duplicate: {}", duplicates.len());
    println!();
    println!("Successful file reads: {}", successes);
    println!("Failed file reads: {}", failures);

    duplicates
}

fn main() {
    let num_files = file_iterator().count();
    println!("Number of files: {}", num_files);

}
