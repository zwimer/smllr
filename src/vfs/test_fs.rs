
// fake filesystem for testing

use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;
use std::rc::Rc;
use std::collections::HashMap;

use super::{File, VFS, MetaData, Inode, DeviceId, FileType};

#[derive(Debug, Clone, Copy)]
pub struct TestMD {
    len: u64,
    creation: SystemTime,
    kind: FileType,
    inode: Inode,
    device: DeviceId,
}

impl MetaData for TestMD {
    fn len(&self) -> u64 {
        self.len
    }
    fn creation_time(&self) -> io::Result<SystemTime> {
        Ok(self.creation)
    }
    fn get_type(&self) -> FileType {
        self.kind
    }
    fn get_inode(&self) -> Inode {
        self.inode
    }
    fn get_device(&self) -> io::Result<DeviceId> {
        Ok(self.device)
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

/*
#[derive(Debug)]
enum VirtElem {
    File { loc: TestFile },
    Dir { loc: PathBuf, contents: Vec<VirtElem>},
    SymLink { loc: PathBuf, target: Box<VirtElem> },
}
*/

/*
impl VirtElem {
    fn get_path(&self) -> PathBuf {
        match self {
            &VirtElem::File { ref loc } => loc.get_path(),
            &VirtElem::Dir { ref loc, .. } => loc.to_owned(),
            &VirtElem::SymLink { ref loc, .. } => loc.to_owned(),
        }
    }
}
*/

#[derive(Debug)]
pub struct TestFileSystem {
    files: HashMap<PathBuf, TestFile>,
    symlinks: HashMap<PathBuf, PathBuf>,
    //root: VirtElem,
}

/*
impl TestFileSystem {
    fn lookup<'a>(&'a self, path: &Path) -> Option<&'a TestFile> {
        let mut current: Option<&VirtElem> = Some(&self.root);
        for part in path {
            match current {
                Some(&VirtElem::File { ref loc }) => if loc.get_path() == path {
                    return Some(loc)
                } else {
                    return None
                },
                Some(&VirtElem::Dir { ref loc, ref contents }) => {
                    if path.starts_with(loc) {
                        current = contents.iter()
                            .find(|&c| path.starts_with(c.get_path()));
                    } else {
                        current = None;
                    }
                },
                Some(&VirtElem::SymLink { ref loc, ref target }) => {
                    if path.starts_with(loc) {
                        current = Some(target);
                    } else {
                        current = None;
                    }
                },
                None => return None,
                _ => unimplemented!()
            }
        }
        None
    }
}
*/

impl VFS for Rc<TestFileSystem> {
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

