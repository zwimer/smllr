
// trait / helper types for filesystem shim

// filesystem object
//  list files in a directory:
//  get file metadata by its path
// file object
//  get metadata
//  get inode
//  get size
//  get filetype


use std::{io, fs, time};
use std::path::{Path, PathBuf};

mod real_fs;
pub use self::real_fs::{RealFileSystem};

mod test_fs;
pub use self::test_fs::{TestFile, TestFileSystem};

// traits

pub trait VFS {
    type FileIter : File;
    
    fn list_dir<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<Box<Iterator<Item=io::Result<Self::FileIter>>>>;

    // follow symlink
    fn get_metadata<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<<Self::FileIter as File>::MD>;
    // information on symlink
    fn get_symlink_metadata<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<<Self::FileIter as File>::MD>;
}

pub trait File {
    type MD : MetaData;
    fn get_inode(&self) -> Inode;
    fn get_path(&self) -> PathBuf;
    fn get_type(&self) -> io::Result<FileType>;
    fn get_metadata(&self) -> io::Result<Self::MD>;
}

pub trait MetaData {
    //fn foo() {}
    fn len(&self) -> u64;
    fn creation_time(&self) -> io::Result<time::SystemTime>;
    fn get_type(&self) -> FileType;
}


// helper types

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    File,
    Dir,
    Symlink,
    Other,
}

impl From<fs::FileType> for FileType {
    fn from(ft: fs::FileType) -> FileType {
        if ft.is_file() {
            FileType::File
        } else if ft.is_dir() {
            FileType::Dir
        } else if ft.is_symlink() {
            FileType::Symlink
        } else {
            // block/char device, fifo, socket, etc depending on os
            FileType::Other
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Inode(u64);
// pub struct DeviceId(u64); // TODO ?

