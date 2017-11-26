#[cfg(test)]
mod test {

    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    use std::rc::Rc;
    use std::path::Path;

    use walker::DirWalker;
    use vfs::TestFileSystem;

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

}
