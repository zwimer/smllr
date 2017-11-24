
// shim around real file system

use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::os::unix::fs::{DirEntryExt, MetadataExt}; // need unix
use std::os::linux::fs::MetadataExt as MetadataExt_linux; // ew
use std::{io, time};
use std::io::Read;

use super::{DeviceId, File, FileType, Inode, MetaData, VFS};
use super::{FirstBytes, Hash, FIRST_K_BYTES};

use md5;

impl MetaData for fs::Metadata {
    fn get_len(&self) -> u64 {
        self.len()
    }
    fn get_creation_time(&self) -> io::Result<time::SystemTime> {
        self.created()
    }
    fn get_type(&self) -> FileType {
        self.file_type().into()
    }
    fn get_inode(&self) -> Inode {
        Inode(self.ino())
    }
    fn get_device(&self) -> io::Result<DeviceId> {
        Ok(DeviceId(self.st_dev()))
    }
}


impl File for DirEntry {
    type MD = fs::Metadata;

    fn get_path(&self) -> PathBuf {
        // warning: heap
        self.path()
    }
    fn get_inode(&self) -> io::Result<Inode> {
        // unix only
        Ok(Inode(self.ino()))
    }
    fn get_type(&self) -> io::Result<FileType> {
        // free/guaranteed on _most_ unixes... not sure when it's not
        // seems to be free on mine
        let ft = self.file_type()?;
        Ok(ft.into())
    }
    fn get_metadata(&self) -> io::Result<fs::Metadata> {
        self.metadata()
    }
    fn get_first_bytes(&self) -> io::Result<FirstBytes> {
        let mut bytes = [0u8; FIRST_K_BYTES];
        let path = self.get_path();
        let mut file = fs::File::open(&path)?;
        file.read(&mut bytes)?;
        Ok(FirstBytes(bytes))
    }
    fn get_hash(&self) -> io::Result<Hash> {
        let path = self.get_path();
        let mut file = fs::File::open(&path)?;
        let mut v = vec![];
        file.read_to_end(&mut v)?;
        Ok(*md5::compute(v))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RealFileSystem;

impl VFS for RealFileSystem {
    type FileIter = DirEntry;

    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<DirEntry>>>> {
        //::std::fs::read_dir(p).map(Box::new) // uhhh why doesn't this work??
        match ::std::fs::read_dir(p) {
            Ok(rd) => Ok(Box::new(rd)),
            Err(e) => Err(e),
        }
    }

    fn get_metadata<P: AsRef<Path>>(&self, p: P) -> io::Result<<Self::FileIter as File>::MD> {
        fs::metadata(p)
    }

    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        fs::symlink_metadata(p)
    }

    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf> {
        fs::read_link(p)
    }
}
