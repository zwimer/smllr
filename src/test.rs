#[cfg(test)]
mod test {

    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    use std::rc::Rc;
    use std::path::{Path, PathBuf};
    use std::collections::HashSet;

    use super::super::{DirWalker, vfs};
    use super::super::vfs::{TestFile, TestFileSystem, TestMD};
    use super::super::FileCataloger;

    fn _enable_logging() {
        LogBuilder::new()
            .filter(None, LogLevelFilter::max())
            .init()
            .unwrap();
    }

    #[test]
    fn empty_fs() {
        let fs = TestFileSystem::new();
        let paths = vec![Path::new("/")];
        let files = DirWalker::new(fs, paths).traverse_all();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn basic_fs() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_file("/alpha");
        }
        let dw = DirWalker::new(fs, vec![Path::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn handle_symlinks() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_file("/alpha");
            // only deal with a target once, omit symlinks
            fs.create_symlink("/beta", "/alpha");
            fs.create_symlink("/gamma", "/alpha");
            // ignore bad symlinks
            fs.create_symlink("/delta", "/_nonexistant");
            // ignore symlink loops
            fs.create_symlink("/x", "/xx");
            fs.create_symlink("/xx", "/x");
        }
        let dw = DirWalker::new(fs, vec![Path::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn dup_detect_poc() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            // add two identical files
            // note that all files passed to FileCataloger must have metadata
            fs.add(
                TestFile::new("/file1")
                    .with_contents(String::from("lorem ipsum"))
                    .with_inode(vfs::Inode(1))
                    .with_metadata(TestMD::new()),
            );
            fs.add(
                TestFile::new("/file2")
                    .with_contents(String::from("lorem ipsum"))
                    .with_inode(vfs::Inode(2))
                    .with_metadata(TestMD::new()),
            );
        }
        let files: HashSet<_> = vec!["/file1", "/file2"].iter().map(PathBuf::from).collect();

        let mut fc = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert_eq!(1, repeats.len());
        let dup = &repeats[0].0;
        assert_eq!(2, dup.len());
        assert!(dup.contains(&PathBuf::from("/file1")));
        assert!(dup.contains(&PathBuf::from("/file2")));
    }

}
