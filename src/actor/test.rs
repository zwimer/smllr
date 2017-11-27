#[cfg(test)]
mod test {

    // verify printing doesn't touch the fs

    // verify deleting works

    // verify linking works

    // verify trying to act on a fs with broken files panics

    use actor::{FileActor, FilePrinter, FileDeleter, Selector};
    use actor::selector::{PathSelect, DateSelect};
    use vfs::{TestFileSystem, TestFile, TestMD};
    use catalog::proxy::Duplicates;

    use std::path::{Path, PathBuf};
    use std::time::{UNIX_EPOCH, Duration};

    // selector tests

    #[test]
    fn select_shortest() {
        let mut fs = TestFileSystem::new();
        fs.create_dir("/");
        fs.create_dir("/w");
        fs.create_dir("/w/x");
        fs.create_dir("/w/x/y");
        fs.create_dir("/w/x/y/z");
        fs.add(TestFile::new("/a"));
        fs.add(TestFile::new("/w/b"));
        fs.add(TestFile::new("/w/x/c"));
        fs.add(TestFile::new("/w/x/y/d"));
        let files = Duplicates(vec!["/a"].iter().map(PathBuf::from).collect());
        let shortest = PathSelect::new(fs).select(&files);
        assert_eq!(shortest, Path::new("/a"));
    }

    #[test]
    fn select_longest() {
        let mut fs = TestFileSystem::new();
        fs.create_dir("/");
        fs.create_dir("/x");
        fs.create_dir("/x/y");
        fs.create_dir("/x/y/z");
        fs.add(TestFile::new("/a"));
        fs.add(TestFile::new("/x/b"));
        fs.add(TestFile::new("/x/y/c"));
        fs.add(TestFile::new("/x/y/z/d"));
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());
        let longest = PathSelect::new(fs).reverse().select(&files);
        assert_eq!(longest, Path::new("/x/y/z/d"));
    }

    /*
    #[test]
    fn select_newest() {
        let mut fs = TestFileSystem::new();
        let time_a = UNIX_EPOCH + Duration::new(1, 0);  // + 1 second
        let time_b = UNIX_EPOCH + Duration::new(2, 0);  // + 2 seconds
        let time_c = UNIX_EPOCH + Duration::new(3, 0);  // + 3 seconds
        let time_d = UNIX_EPOCH + Duration::new(4, 0);  // + 4 seconds
        let md_a = TestMD::new().with_creation_time(time_a);
        let md_b = TestMD::new().with_creation_time(time_b);
        let md_c = TestMD::new().with_creation_time(time_c);
        let md_d = TestMD::new().with_creation_time(time_d);
        fs.create_dir("/");
        fs.create_dir("/x");
        fs.create_dir("/x/y");
        fs.create_dir("/x/y/z");
        fs.add(TestFile::new("/a").with_metadata(md_a));
        fs.add(TestFile::new("/x/b").with_metadata(md_b));
        fs.add(TestFile::new("/x/y/c").with_metadata(md_c));
        fs.add(TestFile::new("/x/y/z/d").with_metadata(md_d));
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());
        let newest = DateSelect::new(fs).select(&files);
        assert_eq!(newest, Path::new("/x/y/z/d"));
    }

    #[test]
    fn select_oldest() {
        let mut fs = TestFileSystem::new();
        let time_a = UNIX_EPOCH + Duration::new(1, 0);  // + 1 second
        let time_b = UNIX_EPOCH + Duration::new(2, 0);  // + 2 seconds
        let time_c = UNIX_EPOCH + Duration::new(3, 0);  // + 3 seconds
        let time_d = UNIX_EPOCH + Duration::new(4, 0);  // + 4 seconds
        let md_a = TestMD::new().with_creation_time(time_a);
        let md_b = TestMD::new().with_creation_time(time_b);
        let md_c = TestMD::new().with_creation_time(time_c);
        let md_d = TestMD::new().with_creation_time(time_d);
        fs.create_dir("/");
        fs.create_dir("/x");
        fs.create_dir("/x/y");
        fs.create_dir("/x/y/z");
        fs.add(TestFile::new("/a").with_metadata(md_a));
        fs.add(TestFile::new("/x/b").with_metadata(md_b));
        fs.add(TestFile::new("/x/y/c").with_metadata(md_c));
        fs.add(TestFile::new("/x/y/z/d").with_metadata(md_d));
        let paths = vec!["/a", "/x/b", "/x/y/c", "/x/y/z/d"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());
        let oldest = DateSelect::new(fs).reverse().select(&files);
        assert_eq!(oldest, Path::new("/a"));
    }
    */

    // actor tests

    #[test]
    fn actor_print() {
        // run `FilePrinter::act()` on a set of duplicates
        // verify the filesystem doesn't change

        let mut fs = TestFileSystem::new();
        fs.create_dir("/");
        fs.add(TestFile::new("/a"));
        fs.create_dir("/x");
        fs.add(TestFile::new("/x/b"));
        fs.add(TestFile::new("/x/c"));
        let paths = vec!["/a", "/x/b", "/x/c"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        let selector = PathSelect::new(fs.clone());
        let mut actor = FilePrinter::new(fs.clone(), selector);
        actor.act(files);
        assert_eq!(5, fs.len());
    }

    #[test]
    fn actor_delete() {
        // run `FileDeleter::act()` on a set of duplicates
        // verify the filesystem doesn't change

        let mut fs = TestFileSystem::new();
        fs.create_dir("/");
        fs.add(TestFile::new("/a").with_metadata(TestMD::new()));
        fs.create_dir("/x");
        fs.add(TestFile::new("/x/b").with_metadata(TestMD::new()));
        fs.add(TestFile::new("/x/c").with_metadata(TestMD::new()));
        let paths = vec!["/a", "/x/b", "/x/c"];
        let files = Duplicates(paths.iter().map(PathBuf::from).collect());

        let selector = PathSelect::new(fs.clone());
        let mut actor = FileDeleter::new(fs.clone(), selector);
        actor.act(files);
        assert_eq!(3, fs.len());
    }
}

