#[cfg(test)]
mod test {

    use std::path::Path;
    use std::ffi::OsString;

    use walker::DirWalker;
    use vfs::TestFileSystem;

    // verify regex blacklist works

    // verify path blacklist works

    // symlink to a parent directory doesn't repeat files

    // symlink targets aren't repeated

    // only a specific directory is included and others are properly omitted

    #[test]
    fn walker_empty_fs() {
        let fs = TestFileSystem::new();
        let paths = vec![Path::new("/")];
        let files = DirWalker::new(fs, &paths).traverse_all();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn walker_basic_fs() {
        let fs = TestFileSystem::new();
        {
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.create_file("/alpha");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn walker_handle_symlinks() {
        let fs = TestFileSystem::new();
        {
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.create_file("/alpha");
            // only deal with a target once, omit symlinks
            fs_.create_symlink("/beta", "/alpha");
            fs_.create_symlink("/gamma", "/alpha");
            // ignore bad symlinks
            fs_.create_symlink("/delta", "/_nonexistant");
            // ignore symlink loops
            fs_.create_symlink("/x", "/xx");
            fs_.create_symlink("/xx", "/x");
            // including a symlink that points to its parent folder
            fs_.create_symlink("/folder", "/");
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
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.create_file("/a.pdf");
            fs_.create_file("/b.txt");
            fs_.create_file("/c.htm");
            fs_.create_file("/d.cpp");
        }
        let dw =
            DirWalker::new(fs, &vec![Path::new("/")]).blacklist_patterns(vec!["/b+", ".*.cpp"]);
        let files = dw.traverse_all();
        assert_eq!(2, files.len());
        assert!(files.contains(Path::new("/a.pdf")));
        assert!(files.contains(Path::new("/c.htm")));
    }

    #[test]
    fn walker_blacklist_folder() {
        // verify files can be blacklisted by their folder
        let fs = TestFileSystem::new();
        {
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.create_dir("/f1");
            fs_.create_dir("/f2");
            fs_.create_dir("/f3");
            fs_.create_dir("/f4");
            fs_.create_file("/f1/a.pdf");
            fs_.create_file("/f2/b.txt");
            fs_.create_file("/f3/c.htm");
            fs_.create_file("/f4/d.cpp");
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
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.create_dir("/f1");
            fs_.create_dir("/f2");
            fs_.create_file("/f1/a.pdf");
            fs_.create_file("/f2/b.txt");
        }
        let dw = DirWalker::new(fs, &vec![Path::new("/f2")]);
        let files = dw.traverse_all();
        assert_eq!(1, files.len());
        assert!(files.contains(Path::new("/f2/b.txt")));
    }

}
