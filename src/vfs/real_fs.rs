
// shim around real file system

use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::os::unix::fs::{DirEntryExt, MetadataExt}; // need unix
use std::os::linux::fs::MetadataExt as MetadataExt_linux; // ew
use std::{io, time};

use super::{DeviceId, File, FileType, Inode, MetaData, VFS};

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

    fn get_file(&self, p: &Path) -> io::Result<Self::FileIter> {
        let dir = p.parent().expect("Called get_file() on root dir");
        match ::std::fs::read_dir(p).expect("Couldn't ls file dir").find(|e| {
            e.as_ref().map(|i| i.path() == p).unwrap_or(false)
        }) {
            Some(f) => Ok(f.unwrap()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file"))
        }
    }
}
