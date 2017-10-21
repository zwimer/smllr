
use std::path::{Path, PathBuf};
use std::{io, env};
//use std::fs::

const FOLLOW_SYMLINKS_DEFAULT: bool = false;

struct DirWalker {
    // files to include/exclude
    directories: Vec<PathBuf>,
    blacklist: Vec<PathBuf>,
    //regex_whitelist: Vec<Pattern>,
    //regex_blacklist: Vec<Pattern>,

    // options
    follow_symlinks: bool,
}


impl DirWalker {
    fn new(dirs: Vec<&Path>) -> DirWalker {
        // if any paths are relative, append them to the current working dir
        // if getting the cwd fails, the whole process should abort
        let abs_paths: io::Result<Vec<_>> = dirs.into_iter().map(|dir| {
            if dir.is_absolute() {
                Ok(dir.to_owned())
            } else {
                info!("Converting `{}` to absolute path", dir.display());
                env::current_dir().map(|cwd| cwd.join(dir))
            }
        }).collect();

        let abs_paths = abs_paths.unwrap_or_else(|e| {
            panic!("Couldn't retrieve current working directory; \
            try using absolute paths or fix your terminal.\n\
            Error: {}", e)
        });

        DirWalker {
            directories: abs_paths,
            blacklist: vec![],
            follow_symlinks: FOLLOW_SYMLINKS_DEFAULT,
        }
    }

}

