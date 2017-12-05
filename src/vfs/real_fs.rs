// shim around real file system so it can be injected into
// DirWalker.

use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::os::unix::fs::{DirEntryExt, MetadataExt}; // need unix
use std::os::linux::fs::MetadataExt as MetadataExt_linux; // ew
use std::{io, time};
use std::io::Read;

use vfs::{File, MetaData, VFS};
use vfs::{DeviceId, FileType, Inode};
//use super::{DeviceId, File, FileType, Inode, MetaData, VFS};
//use super::{FirstBytes, FIRST_K_BYTES};
use helpers::{FirstBytes, FIRST_K_BYTES};
use hash::FileHash;

// Wrap our metadata trait around fs::Metadata.
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
    fn get_first_bytes(&self) -> io::Result<FirstBytes> {
        let mut bytes = [0u8; FIRST_K_BYTES];
        let path = self.get_path();
        let mut file = fs::File::open(&path)?;
        file.read(&mut bytes)?;
        Ok(FirstBytes(bytes))
    }
    fn get_hash<H: FileHash>(&self) -> io::Result<<H as FileHash>::Output> {
        let path = self.get_path();
        let mut file = fs::File::open(&path)?;
        let mut v = vec![];
        file.read_to_end(&mut v)?;
        Ok(H::hash(&v))
    }
}

//Empty struct which represents the 'Real Filesystem'
//all of its 'Member variables' are on the drive.
#[derive(Debug, Clone, Copy)]
pub struct RealFileSystem;

impl VFS for RealFileSystem {
    type FileIter = DirEntry;

    /// Get an iterator over the contents of directory P
    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<DirEntry>>>> {
        match ::std::fs::read_dir(p) {
            Ok(rd) => Ok(Box::new(rd)),
            Err(e) => Err(e),
        }
    }

    /// Look up the metadata for P (follows symlinks)
    fn get_metadata<P: AsRef<Path>>(&self, p: P) -> io::Result<<Self::FileIter as File>::MD> {
        fs::metadata(p)
    }

    /// Look up the metadata for symlink P (don't follow symlinks)
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        fs::symlink_metadata(p)
    }

    /// Resolve symlink P to its target path
    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf> {
        fs::read_link(p)
    }

    /// Look up a File object from its path
    fn get_file(&self, p: &Path) -> io::Result<Self::FileIter> {
        // this is a little hacky for the RealFileSystem
        // the only way to generate a DirEntry is by iterating over a directory
        // so we have to iterate over the parent directory and identify `p`
        let dir = p.parent().expect("Called get_file() on root dir");
        match fs::read_dir(dir)
            .expect("Couldn't ls file dir")
            .find(|e| e.as_ref().map(|i| i.path() == p).unwrap_or(false))
        {
            Some(f) => Ok(f.unwrap()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
        }
    }

    /// Delete a file on the real system
    fn rm_file<P: AsRef<Path>>(&mut self, p: &P) -> io::Result<()> {
        fs::remove_file(p)
    }

    /// Create hard link from `src` to `dst`
    fn make_link(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        fs::hard_link(dst, src)
    }
}
