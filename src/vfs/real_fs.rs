
// shim around real file system

use std::path::{Path, PathBuf};
use std::fs::{self, DirEntry};
use std::os::unix::fs::DirEntryExt; // need unix
use std::{io, time};

use super::{File, VFS, MetaData, Inode, FileType};

#[derive(Debug)]
pub struct RealMD(fs::Metadata);

impl MetaData for RealMD {
    fn len(&self) -> u64 {
        self.0.len()
    }
    fn creation_time(&self) -> io::Result<time::SystemTime> {
        self.0.created()
    }
}

#[derive(Debug)]
pub struct RealFile(DirEntry);

impl File for RealFile {
    type MD = RealMD;

    fn get_path(&self) -> PathBuf {
        // warning: heap
        self.0.path()
    }
    fn get_inode(&self) -> Inode {
        // won't compile for windows
        Inode(self.0.ino())
    }
    fn get_type(&self) -> io::Result<FileType> {
        // free/guaranteed on _most_ unixes... not sure when it's not
        // seems to be free on mine 
        let ft = self.0.file_type()?;
        if ft.is_file() {
            Ok(FileType::File)
        } else if ft.is_dir() {
            Ok(FileType::Dir)
        } else if ft.is_symlink() {
            Ok(FileType::Symlink)
        } else {
            // block/char device, fifo, socket, etc depending on os
            Ok(FileType::Other)
        }
    }
    fn get_metadata(&self) -> io::Result<RealMD> {
        // WARNING always a syscall
        self.0.metadata().map(RealMD)
    }
}

#[derive(Debug)]
pub struct RealFileSystem;

impl VFS for RealFileSystem { 
    type FileIter = DirEntry;

    fn list_dir<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<Box<Iterator<Item=io::Result<DirEntry>>>> 
    {
        //::std::fs::read_dir(p).map(Box::new) // uhhh why doesn't this work??
        match ::std::fs::read_dir(p) {
            Ok(rd) => Ok(Box::new(rd)),
            Err(e) => Err(e)
        }
    }
}

