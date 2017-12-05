#[cfg(test)]
mod test {

    use std::path::Path;
    use std::ffi::OsString;

    use walker::DirWalker;
    use vfs::TestFileSystem;

    #[test]
    fn walker_empty_fs() {
        // verify basic walker/TFS functionality: empty FS is empty
        let fs = TestFileSystem::new();
        let paths = vec![Path::new("/")];
        let files = DirWalker::new(fs, &paths).traverse_all();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn walker_basic_fs() {
        // verify Walker will find a single file on the filesystem
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_file("/alpha");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn walker_handle_symlinks() {
        // test edge cases for symlinks: test data integrity and reselience
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
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
            // including a symlink that points to its parent folder
            fs.create_symlink("/folder", "/");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn walker_blacklist_regex() {
        // verify files can be blacklisted by a regular expression
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_file("/a.pdf");
            fs.create_file("/b.txt");
            fs.create_file("/c.htm");
            fs.create_file("/d.cpp");
        }
        let dw =
            DirWalker::new(fs, &vec![Path::new("/")]).blacklist_patterns(vec!["/b.+", ".*.cpp"]);
        let files = dw.traverse_all();
        println!("{:?}", files);
        assert_eq!(2, files.len());
        assert!(files.contains(Path::new("/a.pdf")));
        assert!(files.contains(Path::new("/c.htm")));
    }

    #[test]
    fn walker_blacklist_folder() {
        // verify files can be blacklisted by their folder
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_dir("/f1");
            fs.create_dir("/f2");
            fs.create_dir("/f3");
            fs.create_dir("/f4");
            fs.create_file("/f1/a.pdf");
            fs.create_file("/f2/b.txt");
            fs.create_file("/f3/c.htm");
            fs.create_file("/f4/d.cpp");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/")])
            .blacklist_folders(vec![&OsString::from("/f1"), &OsString::from("/f2")]);
        let files = dw.traverse_all();
        assert_eq!(2, files.len());
        assert!(files.contains(Path::new("/f3/c.htm")));
        assert!(files.contains(Path::new("/f4/d.cpp")));
    }

    #[test]
    fn walker_ignore_irrelevant_folders() {
        // verify dirwalker only searches in directories it's told to
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_dir("/f1");
            fs.create_dir("/f2");
            fs.create_file("/f1/a.pdf");
            fs.create_file("/f2/b.txt");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/f2")]);
        let files = dw.traverse_all();
        assert_eq!(1, files.len());
        assert!(files.contains(Path::new("/f2/b.txt")));
    }

}
