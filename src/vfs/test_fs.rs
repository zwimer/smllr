
// fake filesystem for testing

use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;
use std::rc::Rc;
use std::collections::HashMap;

use super::{DeviceId, File, FileType, Inode, MetaData, VFS};
use super::{FirstBytes, Hash, FIRST_K_BYTES};
use super::super::ID;

use md5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestMD {
    len: u64,
    creation: SystemTime,
    kind: FileType,
    id: ID,
}

impl MetaData for TestMD {
    fn get_len(&self) -> u64 {
        self.len
    }
    fn get_creation_time(&self) -> io::Result<SystemTime> {
        Ok(self.creation)
    }
    fn get_type(&self) -> FileType {
        self.kind
    }
    fn get_inode(&self) -> Inode {
        Inode(self.id.inode)
    }
    fn get_device(&self) -> io::Result<DeviceId> {
        Ok(DeviceId(self.id.dev))
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct TestFile {
    path: PathBuf,
    contents: Option<String>,
    kind: FileType,
    inode: Inode,
    metadata: Option<TestMD>,
}

impl File for TestFile {
    type MD = TestMD;

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
    fn get_inode(&self) -> io::Result<Inode> {
        Ok(self.inode)
    }
    fn get_type(&self) -> io::Result<FileType> {
        Ok(self.kind)
    }
    fn get_metadata(&self) -> io::Result<TestMD> {
        self.metadata
            .ok_or(io::Error::new(io::ErrorKind::Other, "No MD"))
    }
    fn get_first_bytes(&self) -> io::Result<FirstBytes> {
        if let Some(ref cont) = self.contents {
            let mut bytes = [0u8; FIRST_K_BYTES];
            for (c,b) in cont.bytes().zip(bytes.iter_mut()) {
                *b = c;
            }
            Ok(FirstBytes(bytes))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No contents set"))
        }
    }
    fn get_hash(&self) -> io::Result<Hash> {
        if let Some(ref cont) = self.contents {
            Ok(*md5::compute(cont))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No contents set"))
        }
    }
}


#[derive(Debug)]
pub struct TestFileSystem {
    files: HashMap<PathBuf, TestFile>,
    symlinks: HashMap<PathBuf, (TestFile, PathBuf)>,
}

impl TestFileSystem {
    // helpers
    fn get_next_inode(&self) -> Inode {
        Inode((self.files.len() + self.symlinks.len()) as u64)
    }
    fn create_regular(&mut self, path: &Path, kind: FileType) {
        let inode = self.get_next_inode();
        let md = TestMD {
            len: 0,
            creation: SystemTime::now(),
            kind,
            id: ID {
                inode: inode.0,
                dev: 0,
            },
        };
        let tf = TestFile {
            path: path.to_owned(),
            kind,
            inode,
            contents: None,
            metadata: Some(md),
        };
        self.files.insert(path.to_owned(), tf);
    }

    // insert into
    pub fn new() -> Rc<Self> {
        Rc::new(TestFileSystem {
            files: HashMap::new(),
            symlinks: HashMap::new(),
        })
    }
    pub fn create_file<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::File);
    }
    pub fn create_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::Dir);
    }
    pub fn create_symlink<P: AsRef<Path>>(&mut self, path: P, target: P) {
        let tf = TestFile {
            path: path.as_ref().to_owned(),
            kind: FileType::Symlink,
            inode: self.get_next_inode(),
            contents: None,
            metadata: None,
        };
        let val = (tf, target.as_ref().to_owned());
        self.symlinks.insert(path.as_ref().to_owned(), val);
    }

    // getters
    fn lookup<'a>(&'a self, path: &Path) -> io::Result<&'a TestFile> {
        if let Some(tf) = self.files.get(path) {
            Ok(tf)
        } else {
            // traverse the symlink chain
            let mut cur = self.symlinks.get(path);
            let mut seen: Vec<&Path> = vec![]; // SystemTime isn't Hash
            while let Some(c) = cur {
                if seen.contains(&c.1.as_path()) {
                    // infinite symlink loop
                    return Err(io::Error::from_raw_os_error(40));
                } else {
                    seen.push(&c.1);
                    cur = self.symlinks.get(&c.1);
                }
            }
            Err(io::Error::new(io::ErrorKind::NotFound, "No such file"))
        }
    }
}

impl VFS for Rc<TestFileSystem> {
    type FileIter = TestFile;

    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<TestFile>>>> {
        let mut v = vec![];
        for (path, file) in &self.files {
            let parent = path.parent();
            if parent == Some(p.as_ref()) || parent.is_none() {
                v.push(Ok(file.clone()));
            }
        }
        for (src, &(ref file, ref _dst)) in &self.symlinks {
            if src.parent() == Some(p.as_ref()) {
                v.push(Ok(file.clone()));
            }
        }
        Ok(Box::new(v.into_iter()))
    }

    fn get_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<<Self::FileIter as File>::MD> {
        // FileType cannot be symlink
        match self.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match self.symlinks.get(path.as_ref()) {
                Some(&(_, ref p)) => self.lookup(p).and_then(|f| f.get_metadata()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        // FileType can be symlink
        match self.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match self.symlinks.get(path.as_ref()) {
                Some(&(ref f, _)) => f.get_metadata(),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        match self.symlinks.get(path.as_ref()) {
            Some(&(_, ref p)) => Ok(p.to_owned()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
        }
    }
}
