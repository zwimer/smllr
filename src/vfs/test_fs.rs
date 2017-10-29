
// fake filesystem for testing

use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;

use super::{File, VFS, MetaData, Inode, FileType};

#[derive(Debug, Clone, Copy)]
pub struct TestMD {
    len: u64,
    creation: SystemTime,
}

impl MetaData for TestMD {
    fn len(&self) -> u64 {
        self.len
    }
    fn creation_time(&self) -> io::Result<SystemTime> {
        Ok(self.creation)
    }
    fn get_type(&self) -> FileType {
        unimplemented!()
    }
    fn get_inode(&self) -> Inode {
        unimplemented!()
    }
}


#[derive(Debug, Clone)]
pub struct TestFile {
    path: String,
    contents: String,
    kind: FileType,
    inode: Inode,
    metadata: Option<TestMD>,
}

impl File for TestFile {
    type MD = TestMD;

    fn get_path(&self) -> PathBuf {
        Path::new(&self.path).to_owned()
    }
    fn get_inode(&self) -> io::Result<Inode> {
        Ok(self.inode)
    }
    fn get_type(&self) -> io::Result<FileType> {
        Ok(self.kind)
    }
    fn get_metadata(&self) -> io::Result<TestMD> {
        self.metadata.ok_or(io::Error::new(io::ErrorKind::Other, "No MD"))
    }
}

#[derive(Debug)]
enum VirtElem {
    File(TestFile),
    Dir(Vec<VirtElem>),
    SymLink(PathBuf),
}

#[derive(Debug)]
pub struct TestFileSystem {
    root: VirtElem,
}

impl VFS for TestFileSystem {
    type FileIter = TestFile;

    fn list_dir<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<Box<Iterator<Item=io::Result<TestFile>>>> 
    {
        unimplemented!()
    }

    fn get_metadata<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<<Self::FileIter as File>::MD> 
    {
        unimplemented!()
    }

    fn get_symlink_metadata<P: AsRef<Path>>(&self, p: P) 
        -> io::Result<<Self::FileIter as File>::MD>
    {
        unimplemented!()
    }

    //fn resolve_path<P: AsRef<Path>>(&self, p: P) -> io::Result<Self::FileIter> {
    //    unimplemented!()
    //}
    fn read_link<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf> {
        unimplemented!()
    }
}

