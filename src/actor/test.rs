#[cfg(test)]
mod test {

    use actor::{FileActor, FileDeleter, FileLinker, FilePrinter};
    use actor::selector::{DateSelect, PathSelect, Selector};
    use vfs::{TestFile, TestFileSystem, TestMD};
    use catalog::proxy::Duplicates;

    use std::path::{Path, PathBuf};
    use std::time::{Duration, UNIX_EPOCH};

    // selector tests

    #[test]
    fn select_shortest() {
        // select the file closest to the root
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
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
        // select the file farthest from to the root
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_dir("/x");
            fs.create_dir("/x/y");
            fs.create_dir("/x/y/z");
            fs.add(TestFile::new("/a"));
            fs.add(TestFile::new("/x/b"));
            fs.add(TestFile::new("/x/y/c"));
            fs.add(TestFile::new("/x/y/z/d"));
        }
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());
        let mut selector = PathSelect::new(fs);
        selector.reverse();
        let longest = selector.select(&files);
        assert_eq!(longest, Path::new("/x/y/z/d"));
    }

    #[test]
    fn select_newest() {
        // select the file most recently modified
        let fs = TestFileSystem::new();
        let time_a = UNIX_EPOCH + Duration::new(1, 0); // + 1 second
        let time_b = UNIX_EPOCH + Duration::new(2, 0); // + 2 seconds
        let time_c = UNIX_EPOCH + Duration::new(3, 0); // + 3 seconds
        let time_d = UNIX_EPOCH + Duration::new(4, 0); // + 4 seconds
        let md_a = TestMD::new().with_creation_time(time_a);
        let md_b = TestMD::new().with_creation_time(time_b);
        let md_c = TestMD::new().with_creation_time(time_c);
        let md_d = TestMD::new().with_creation_time(time_d);
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_dir("/x");
            fs.create_dir("/x/y");
            fs.create_dir("/x/y/z");
            fs.add(TestFile::new("/a").with_metadata(md_a));
            fs.add(TestFile::new("/x/b").with_metadata(md_b));
            fs.add(TestFile::new("/x/y/c").with_metadata(md_c));
            fs.add(TestFile::new("/x/y/z/d").with_metadata(md_d));
        }
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());
        let newest = DateSelect::new(fs).select(&files);
        assert_eq!(newest, Path::new("/x/y/z/d"));
    }

    #[test]
    fn select_oldest() {
        // select the file least recently modified
        let fs = TestFileSystem::new();
        let time_a = UNIX_EPOCH + Duration::new(1, 0); // + 1 second
        let time_b = UNIX_EPOCH + Duration::new(2, 0); // + 2 seconds
        let time_c = UNIX_EPOCH + Duration::new(3, 0); // + 3 seconds
        let time_d = UNIX_EPOCH + Duration::new(4, 0); // + 4 seconds
        let md_a = TestMD::new().with_creation_time(time_a);
        let md_b = TestMD::new().with_creation_time(time_b);
        let md_c = TestMD::new().with_creation_time(time_c);
        let md_d = TestMD::new().with_creation_time(time_d);
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.create_dir("/x");
            fs.create_dir("/x/y");
            fs.create_dir("/x/y/z");
            fs.add(TestFile::new("/a").with_metadata(md_a));
            fs.add(TestFile::new("/x/b").with_metadata(md_b));
            fs.add(TestFile::new("/x/y/c").with_metadata(md_c));
            fs.add(TestFile::new("/x/y/z/d").with_metadata(md_d));
        }
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        let mut selector = DateSelect::new(fs.clone());
        selector.reverse();
        let oldest = selector.select(&files);
        assert_eq!(oldest, Path::new("/a"));
    }

    // actor tests

    #[test]
    fn actor_print() {
        // run `FilePrinter::act()` on a set of duplicates
        // verify the filesystem doesn't change

        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.add(TestFile::new("/a"));
            fs.create_dir("/x");
            fs.add(TestFile::new("/x/b"));
            fs.add(TestFile::new("/x/c"));
        };
        let paths = vec!["/a", "/x/b", "/x/c"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        let selector = PathSelect::new(fs.clone());
        let mut actor = FilePrinter::new(selector);
        actor.act(files);
        assert_eq!(5, fs.borrow().num_elements());
    }

    #[test]
    fn actor_delete() {
        // run `FileDeleter::act()` on a set of duplicates
        // verify the filesystem only has one file left

        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.add(TestFile::new("/a").with_metadata(TestMD::new()));
            fs.create_dir("/x");
            fs.add(TestFile::new("/x/b").with_metadata(TestMD::new()));
            fs.add(TestFile::new("/x/c").with_metadata(TestMD::new()));
        };
        let paths = vec!["/a", "/x/b", "/x/c"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        let selector = PathSelect::new(fs.clone());
        let mut actor = FileDeleter::new(fs.clone(), selector);
        actor.act(files);
        assert_eq!(3, fs.borrow().num_elements());
    }

    #[test]
    fn actor_link() {
        // run `FileLinker::act()` on a set of duplicates
        // verify the filesystem only has links to one file

        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/"); // inode #0
            fs.add(
                TestFile::new("/a")
                    .with_inode(1)
                    .with_metadata(TestMD::new()),
            );
            fs.add(
                TestFile::new("/b")
                    .with_inode(2)
                    .with_metadata(TestMD::new()),
            );
            fs.add(
                TestFile::new("/c")
                    .with_inode(3)
                    .with_metadata(TestMD::new()),
            );
        };
        let paths = vec!["/a", "/b", "/c"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        // currently all files are identical and distinct
        // remember that the root dir counts and has an inode
        assert_eq!(4, fs.borrow().num_elements(), "sanity check");
        assert_eq!(4, fs.borrow().num_inodes(), "sanity check");

        let selector = PathSelect::new(fs.clone());
        let mut actor = FileLinker::new(fs.clone(), selector);
        actor.act(files);

        // after acting, all files should have the same inode
        assert_eq!(4, fs.borrow().num_elements());
        assert_eq!(2, fs.borrow().num_inodes());
    }
}
