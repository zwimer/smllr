#[cfg(test)]
mod test {

    use vfs::{TestFile, TestFileSystem, TestMD};
    use catalog::FileCataloger;
    use hash::{Md5Sum, Sha3Sum};

    use std::path::PathBuf;
    use std::collections::HashSet;

    #[test]
    fn dup_all_unique() {
        // completely distinct files should not be flagged as duplicates
        // they share no criteria in common (inode/size/hash/beginning)

        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.add(
                TestFile::new("/a")
                    .with_contents(String::from("A"))
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs.add(
                TestFile::new("/b")
                    .with_contents(String::from("BB"))
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
            fs.add(
                TestFile::new("/c")
                    .with_contents(String::from("CCC"))
                    .with_metadata(TestMD::new())
                    .with_inode(3),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b", "/c"].iter().map(PathBuf::from).collect();

        let mut fc: FileCataloger<_, Sha3Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert!(repeats.is_empty());
    }

    #[test]
    fn dup_test_same_size() {
        // files with the same length but different contents
        // files should not be flagged as duplicates
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            fs.add(
                TestFile::new("/a")
                    .with_contents(String::from("AAAA"))
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs.add(
                TestFile::new("/b")
                    .with_contents(String::from("BBBB"))
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
            fs.add(
                TestFile::new("/c")
                    .with_contents(String::from("CCCC"))
                    .with_metadata(TestMD::new())
                    .with_inode(3),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b", "/c"].iter().map(PathBuf::from).collect();

        let mut fc: FileCataloger<_, Md5Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert!(repeats.is_empty());
    }

    #[test]
    fn dup_test_same_start() {
        // files with the same length and first k bytes but different contents
        // files should not be flagged as duplicates
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            let start: String = ::std::iter::repeat('A').take(4096).collect();
            fs.add(
                TestFile::new("/a")
                    .with_contents(format!("{}_a", start))
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs.add(
                TestFile::new("/b")
                    .with_contents(format!("{}_b", start))
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
            fs.add(
                TestFile::new("/c")
                    .with_contents(format!("{}_c", start))
                    .with_metadata(TestMD::new())
                    .with_inode(3),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b", "/c"].iter().map(PathBuf::from).collect();

        let mut fc: FileCataloger<_, Md5Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert!(repeats.is_empty());
    }

    #[test]
    fn dup_test_same_contents() {
        // unlinked files with the same contents should be flagged as duplicates
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            let contents: String = ::std::iter::repeat('A').take(4096).collect();
            fs.add(
                TestFile::new("/a")
                    .with_contents(contents.clone())
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs.add(
                TestFile::new("/b")
                    .with_contents(contents.clone())
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
            fs.add(
                TestFile::new("/c")
                    .with_contents(contents)
                    .with_metadata(TestMD::new())
                    .with_inode(3),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b", "/c"].iter().map(PathBuf::from).collect();

        let mut fc: FileCataloger<_, Md5Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert_eq!(1, repeats.len());
        let dups = &repeats[0].0;
        assert_eq!(3, dups.len());
        assert!(dups.contains(&PathBuf::from("/a")));
        assert!(dups.contains(&PathBuf::from("/b")));
    }

    #[test]
    fn dup_test_hard_links() {
        // hard links to the same file should be flagged as duplicates
        // even if they (somehow) have different contents
        let fs = TestFileSystem::new();
        {
            let mut fs = fs.borrow_mut();
            fs.create_dir("/");
            // note that all test files passed to FileCataloger must have metadata
            fs.add(
                TestFile::new("/a")
                    .with_inode(1)
                    .with_contents(String::from("AAAA"))
                    .with_metadata(TestMD::new()),
            );
            fs.add(
                TestFile::new("/b")
                    .with_inode(1)
                    .with_contents(String::from("BBBB"))
                    .with_metadata(TestMD::new()),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b"].iter().map(PathBuf::from).collect();

        let mut fc: FileCataloger<_, Md5Sum> = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert_eq!(1, repeats.len());
        let dup = &repeats[0].0;
        assert_eq!(2, dup.len());
        assert!(dup.contains(&PathBuf::from("/a")));
        assert!(dup.contains(&PathBuf::from("/b")));
    }

}
