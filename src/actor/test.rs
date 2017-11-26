#[cfg(test)]
mod test {

    // verify printing doesn't touch the fs

    // verify deleting works

    // verify linking works

    // verify trying to act on a fs with broken files panics

    use actor::{Selector, FilePrinter};
    use actor::selector::PathSelect;
    use vfs::{TestFileSystem, TestFile};
    use catalog::proxy::Duplicates;

    use std::rc::Rc;
    use std::path::{Path, PathBuf};

    #[test]
    fn actor_print_readonly() {
        //let selector = 
    }

    #[test]
    fn select_shortest() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_dir("/w");
            fs.create_dir("/w/x");
            fs.create_dir("/w/x/y");
            fs.create_dir("/w/x/y/z");
            fs.add(TestFile::new("/a"));
            fs.add(TestFile::new("/w/b"));
            fs.add(TestFile::new("/w/x/c"));
            fs.add(TestFile::new("/w/x/y/d"));
        }
        let files = Duplicates(vec!["/a"].iter().map(PathBuf::from).collect());
        let shortest = PathSelect::new(fs).select(&files);
        assert_eq!(shortest, Path::new("/a"));
    }

    #[test]
    fn select_longest() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_dir("/x");
            fs.create_dir("/x/y");
            fs.create_dir("/x/y/z");
            fs.add(TestFile::new("/a"));
            fs.add(TestFile::new("/x/b"));
            fs.add(TestFile::new("/x/y/c"));
            fs.add(TestFile::new("/x/y/z/d"));
        }
        let files = Duplicates(vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"]
                               .iter().map(PathBuf::from).collect());
        let longest = PathSelect::new(fs).reverse().select(&files);
        assert_eq!(longest, Path::new("/x/y/z/d"));
    }

    #[test]
    fn select_newest() {}

    #[test]
    fn select_oldest() {}
}
