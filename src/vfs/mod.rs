
// trait / helper types for filesystem shim

// filesystem object
//  list files in a directory:
//  get file metadata by its path
// file object
//  get metadata
//  get inode
//  get size
//  get filetype


use std::{io,time};
use std::path::{Path, PathBuf};

mod real_fs;
pub use self::real_fs::{RealFile, RealFileSystem};

mod test_fs;
pub use self::test_fs::{TestFile, TestFileSystem};

// traits

trait VFS {
    type FileIter;
    fn list_dir<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<Box<Iterator<Item=io::Result<Self::FileIter>>>>;
}

trait File {
    type MD : MetaData;
    fn get_inode(&self) -> Inode;
    fn get_path(&self) -> PathBuf;
    fn get_type(&self) -> io::Result<FileType>;
    fn get_metadata(&self) -> io::Result<Self::MD>;
}

trait MetaData {
    fn len(&self) -> u64;
    fn creation_time(&self) -> io::Result<time::SystemTime>;
}


// helper types

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    File,
    Dir,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct Inode(u64);
// pub struct DeviceId(u64); // TODO ?

