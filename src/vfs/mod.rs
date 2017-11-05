
// trait / helper types for filesystem shim


use std::{fs, io, time};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

mod real_fs;
pub use self::real_fs::RealFileSystem;

mod test_fs;
pub use self::test_fs::{TestFile, TestFileSystem};

// traits

pub trait VFS: Clone + Debug {
    type FileIter: File;

    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<Self::FileIter>>>>;

    // follow symlink
    fn get_metadata<P: AsRef<Path>>(&self, p: P) -> io::Result<<Self::FileIter as File>::MD>;
    // information on symlink
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<<Self::FileIter as File>::MD>;

    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf>;
}

pub trait File: Debug {
    type MD: MetaData;
    fn get_inode(&self) -> io::Result<Inode>;
    fn get_path(&self) -> PathBuf;
    fn get_type(&self) -> io::Result<FileType>;
    fn get_metadata(&self) -> io::Result<Self::MD>;
}

pub trait MetaData: Debug {
    fn get_len(&self) -> u64;
    fn get_creation_time(&self) -> io::Result<time::SystemTime>;
    fn get_type(&self) -> FileType;
    fn get_inode(&self) -> Inode;
    fn get_device(&self) -> io::Result<DeviceId>;
}


// helper types

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Inode(u64);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceId(u64);
