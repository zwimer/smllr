#[cfg(test)]
mod test {

    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    use std::rc::Rc;
    use std::path::Path;

    use super::super::DirWalker;
    use super::super::vfs::{TestFileSystem};

    fn _enable_logging() {
        LogBuilder::new().filter(None, LogLevelFilter::max()).init().unwrap();
    }

    #[test]
    fn empty_fs() {
        let fs = TestFileSystem::new();
        let paths = vec![Path::new("/")];
        let mut dw = DirWalker::new(fs, paths);
        let count: usize = dw.traverse_all();
        assert_eq!(count, 0);
    }

    #[test]
    fn basic_fs() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_file("/alpha");
        }
        let mut dw = DirWalker::new(fs, vec![Path::new("/")]);
        let count: usize = dw.traverse_all();
        assert_eq!(count, 1);
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
        let mut dw = DirWalker::new(fs, vec![Path::new("/")]);
        let count = dw.traverse_all();
        assert_eq!(count, 1);
    }

}
