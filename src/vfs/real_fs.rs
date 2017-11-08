// shim around real file system so it can be injected into
// DirWalker.

use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::os::unix::fs::{DirEntryExt, MetadataExt}; // need unix
use std::os::linux::fs::MetadataExt as MetadataExt_linux; // ew
use std::{io, time};

use super::{DeviceId, File, FileType, Inode, MetaData, VFS};

//Wrap our metadata trait around fs::Metadata.
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

// wrapping our File interface around the stdd DirEntry.
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

//Empty struct which represents the 'Real Filesystem'
//all of its 'Member variables' are on the drive.
#[derive(Debug, Clone, Copy)]
pub struct RealFileSystem;

impl VFS for RealFileSystem {
    type FileIter = DirEntry;
    /// get an iterator over the contents of directory P
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
    /// get the metadata for P
    fn get_metadata<P: AsRef<Path>>(&self, p: P) -> io::Result<<Self::FileIter as File>::MD> {
        fs::metadata(p)
    }
    /// get the metadata for symlink P
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        fs::symlink_metadata(p)
    }
    /// resolve symlink P to its target path
    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf> {
        fs::read_link(p)
    }
}
