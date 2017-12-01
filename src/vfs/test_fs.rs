// mock filesystem for testing

use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{self, SystemTime};
use std::collections::{HashMap, HashSet};
//RUST NOTE: `super` means up a module (often up a directory)
use vfs::{DeviceId, File, FileType, Inode, MetaData, VFS};
use super::{FirstBytes, FIRST_K_BYTES};
use super::super::ID;
use hash::{FileHash, Hash};

/// `TestMD` is the mock metadata struct.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestMD {
    len: u64,
    creation: SystemTime,
    kind: FileType,
    id: ID,
}

//implementation of the MetaData trait for testMD.
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

// TestMD must be easy to make and also customize for unit testing
// We provide a series of chainable setters to easily construct test objects
// e.g. `TestMD::new().with_len(4096).with_id(42)`
impl TestMD {
    pub fn new() -> Self {
        TestMD {
            len: 0,
            creation: SystemTime::now(),
            kind: FileType::File,
            id: ID { dev: 0, inode: 0 },
        }
    }
    pub fn with_len(mut self, n: u64) -> Self {
        self.len = n;
        self
    }
    pub fn with_creation_time(mut self, t: SystemTime) -> Self {
        self.creation = t;
        self
    }
    pub fn with_kind(mut self, k: FileType) -> Self {
        self.kind = k;
        self
    }
    pub fn with_id(mut self, id: ID) -> Self {
        self.id = id;
        self
    }
}

/// `TestFile` denotes a mockfile.
/// Note that we are mocking the linux-style filesystem
/// where many things are 'files', including directories,
/// links, devices (/dev/sda might be familair).
#[derive(Debug, Clone, PartialEq)]
pub struct TestFile {
    path: PathBuf,
    contents: Option<String>,
    kind: FileType,
    inode: Inode,
    metadata: Option<TestMD>,
}

// build up a File object for mock testing
// Chainable setters to easily construct test objects
// e.g. `TestFile::new().with_kind(FileType::Dir).with_inode(42)`
impl TestFile {
    pub fn new(s: &str) -> Self {
        TestFile {
            path: PathBuf::from(s),
            contents: None,
            kind: FileType::File,
            inode: Inode(0),
            metadata: None,
        }
    }
    pub fn with_contents(mut self, c: String) -> Self {
        if let Some(ref mut md) = self.metadata {
            md.len = c.len() as u64;
        }
        self.contents = Some(c);
        self
    }
    pub fn with_kind(mut self, k: FileType) -> Self {
        if let Some(ref mut md) = self.metadata {
            md.kind = k;
        }
        self.kind = k;
        self
    }
    pub fn with_inode(mut self, i: u64) -> Self {
        if let Some(ref mut md) = self.metadata {
            md.id.inode = i;
        }
        self.inode = Inode(i);
        self
    }
    pub fn with_metadata(mut self, mut md: TestMD) -> Self {
        // fix filetype discrepancy
        if self.kind != FileType::File {
            md.kind = self.kind;
        } else if md.kind != FileType::File {
            self.kind = md.kind;
        }
        // fix len discrepancy
        if let Some(ref c) = self.contents {
            md.len = c.len() as u64;
        } else if md.len != 0 {
            // for now do nothing
            // it is okay for `len` to be >0 and `contents` to be empty
            //let contents = ::std::iter::repeat('?').take(md.len as usize).collect();
            //self.contents = Some(contents);
        }
        // fix inode discrepancy
        if self.inode.0 != 0 {
            md.id.inode = self.inode.0;
        } else if md.id.inode != 0 {
            self.inode.0 = md.id.inode;
        }
        self.metadata = Some(md);
        self
    }
}

/// Implementation of the File trait for `TestFile`
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
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No MD"))
    }
    fn get_first_bytes(&self) -> io::Result<FirstBytes> {
        // read the first K bytes of the file
        // if the file is less than K bytes, the remaining bytes are treated as zeros
        if let Some(ref cont) = self.contents {
            let mut bytes = [0u8; FIRST_K_BYTES];
            for (c, b) in cont.bytes().zip(bytes.iter_mut()) {
                *b = c;
            }
            Ok(FirstBytes(bytes))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No contents set"))
        }
    }
    fn get_hash<H: FileHash>(&self, hasher: &H) -> io::Result<Hash> {
        // hash the contents of the file
        if let Some(ref cont) = self.contents {
            Ok(hasher.hash(cont.as_bytes()))
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "No contents set"))
        }
    }
}

/// `TestFileSystem` denotes a Mock Filesystem we use instead of risking
/// our own data or dealing with the actual filesystem
#[derive(Debug)]
pub struct TestFileSystem {
    files: HashMap<PathBuf, TestFile>,
    symlinks: HashMap<PathBuf, (TestFile, PathBuf)>,
}

impl TestFileSystem {
    // private helper functions:
    // gets the number of files on the mock system.
    // The name denotes its use when adding a new inode,
    // as sequentially, they are numbered 0, 1, ...
    // Ergo with N inodes, the next inode will be
    // given the id N.
    fn get_next_inode(&self) -> Inode {
        Inode((self.files.len() + self.symlinks.len()) as u64)
    }
    // Creates a new MockFile with FileType kind and a Path of path
    // Not used to create a new symlink.
    fn create_regular(&mut self, path: &Path, kind: FileType) {
        let inode = self.get_next_inode();
        // Create the metadata for the file
        let md = TestMD {
            len: 0,
            creation: time::UNIX_EPOCH,
            kind,
            id: ID {
                inode: inode.0,
                dev: 0,
            },
        };
        // Create the File.
        let tf = TestFile {
            path: path.to_owned(),
            kind,
            inode,
            contents: None,
            metadata: Some(md),
        };
        // Add the file to the filesystem.
        self.files.insert(path.to_owned(), tf);
    }

    /// constructor: initializes self.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TestFileSystem {
            files: HashMap::new(),
            symlinks: HashMap::new(),
        }))
    }
    /// get size
    pub fn num_elements(&self) -> usize {
        self.files.len() + self.symlinks.len()
    }
    /// get number of unique inodes
    pub fn num_inodes(&self) -> usize {
        let inodes: HashSet<_> = self.files
            .iter()
            .map(|f| f.1.get_inode().unwrap())
            .collect();
        inodes.len()
    }
    /// Creates a new file at path. Anologous to '$touch path'
    pub fn create_file<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::File);
    }
    /// Creates a new directory with path. Anologus to '$mkdir path'
    pub fn create_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::Dir);
    }
    /// Creates a new symlink from path to target. analogous to
    /// `ln -s -t target path`
    pub fn create_symlink<P: AsRef<Path>>(&mut self, path: P, target: P) {
        // Create the symlink file.
        let tf = TestFile {
            path: path.as_ref().to_owned(),
            kind: FileType::Symlink,
            inode: self.get_next_inode(),
            contents: None,
            metadata: None,
        };
        // add the symlink to the filesystem.
        let val = (tf, target.as_ref().to_owned());
        self.symlinks.insert(path.as_ref().to_owned(), val);
    }
    /// Register a new file
    pub fn add(&mut self, tf: TestFile) {
        self.files.insert(tf.path.to_owned(), tf);
    }

    // getters for the Mock Filesystem.
    // RUST SYNTAX: <'a> is a lifetime paramater. Lifetimes are pretty
    // unique to rust; essentially they are used to pass the parent
    // through so they are invalidated when the parent is.

    /// Resolves the path into a TestFile
    fn lookup<'a>(&'a self, path: &Path) -> io::Result<&'a TestFile> {
        if let Some(tf) = self.files.get(path) {
            Ok(tf)
        } else {
            // traverse the symlink chain
            let mut cur = self.symlinks.get(path);
            // `seen` can't be a Hash table because SystemTime isn't Hash
            let mut seen: Vec<&Path> = vec![];
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

// Implementation of the VFS interface for the whole of the Mock File System.
impl VFS for Rc<RefCell<TestFileSystem>> {
    type FileIter = TestFile;

    /// VFS::list_dir(p)  gets an iterator over the contents of p.
    fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<TestFile>>>> {
        let mut v = vec![];
        let fs = self.borrow();
        // collect all files which are children of p
        let is_root = p.as_ref().components().count() == 1;
        for (path, file) in &fs.files {
            if path.parent() == Some(p.as_ref()) || (is_root && path.components().count() == 2) {
                // include a file if it's parent is in `p`
                // or if `p` is the root and `path` is 1 level down
                v.push(Ok(file.clone()));
            }
        }
        // collect all symlinks which are children of p
        for (src, &(ref file, ref _dst)) in &fs.symlinks {
            if src.parent() == Some(p.as_ref()) || p.as_ref().parent().is_none() {
                v.push(Ok(file.clone()));
            }
        }
        // return the iterator.
        Ok(Box::new(v.into_iter()))
    }

    //RUST NOTE: match is roughly equivlent to the c's 'switch'.
    // match expr {
    //     expr1 => block,
    //     expr2 => block,
    // }
    // is equivlent to
    // switch (expr) {
    //     case expr1:
    //         block
    //     case expr2:
    //         block
    //}
    //
    // The '_' expresion when used in match is equivelent to default in c
    //
    //Match also supports deconstructing and binding. see
    // https://rustbyexample.com/flow_control/match.html
    // for more information.

    /// VFS::get_metadata gets the Metadata of Path
    /// FileType of path cannot be symlink; they are handled diffrently; use
    /// VFS::get_symlink_metadata for symlinks
    fn get_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<<Self::FileIter as File>::MD> {
        let fs = self.borrow();
        match fs.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match fs.symlinks.get(path.as_ref()) {
                Some(&(_, ref p)) => fs.lookup(p).and_then(|f| f.get_metadata()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    /// VFS::get_symlink_metadata(p) gets the metadata for symlink p.
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        let fs = self.borrow();
        match fs.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match fs.symlinks.get(path.as_ref()) {
                Some(&(ref f, _)) => f.get_metadata(),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    /// VFS::read_link(p) resolves symlink at path p to the path its pointing to
    /// or gives an error if the link is broken.
    fn read_link<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        match self.borrow().symlinks.get(path.as_ref()) {
            Some(&(_, ref p)) => Ok(p.to_owned()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
        }
    }

    fn get_file(&self, p: &Path) -> io::Result<Self::FileIter> {
        match self.borrow().files.get(p) {
            Some(f) => Ok(f.to_owned()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
        }
    }

    fn rm_file<P: AsRef<Path>>(&mut self, p: &P) -> io::Result<()> {
        let mut fs = self.borrow_mut();
        match fs.files.remove(p.as_ref()) {
            Some(_) => Ok(()),
            None => Err(io::Error::new(io::ErrorKind::Other, "Couldn't delete file")),
        }
    }

    // create a hard link
    fn make_link(&mut self, src: &Path, dst: &Path) -> io::Result<()> {
        let mut fs = self.borrow_mut();
        let old_md = fs.files
            .get(dst)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No dst file"))?
            .get_metadata()?;
        // need to know the old inode so the new can have the same
        let old_inode = old_md.get_inode();
        let old_device = old_md.get_device()?;

        // verify new file is going to be
        let new_dir = src.parent().unwrap(); // can't be root
        let new_device = fs.files
            .get(new_dir)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No src md"))?
            .get_metadata()?
            .get_device()?;

        if old_device != new_device {
            // can't make a hard link across devices (on most filesystems)
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Cannot make hard link across filesystems",
            ));
        }

        let name = src.to_str().expect("invalid unicode link name");
        fs.files.insert(
            src.to_path_buf(),
            TestFile::new(name).with_inode(old_inode.0),
        );
        Ok(())
    }
}
