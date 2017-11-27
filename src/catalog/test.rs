#[cfg(test)]
mod test {

    use vfs::{TestFile, TestFileSystem, TestMD};

    use catalog::FileCataloger;

    use std::path::PathBuf;
    use std::collections::HashSet;

    // verify files w/ the same size but different values aren't the same

    // verify files w/ the same size and first k bytes aren't the same

    // verify identical files are matched

    // hard links / across drives ?

    #[test]
    fn dup_test_same_size() {
        // files with the same length but different contents
        // files should not be flagged as duplicates
        let fs = TestFileSystem::new();
        {
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            fs_.add(
                TestFile::new("/a")
                    .with_contents(String::from("AAAA"))
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs_.add(
                TestFile::new("/b")
                    .with_contents(String::from("BBBB"))
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b"].iter().map(PathBuf::from).collect();

        let mut fc = FileCataloger::new(fs);
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
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            let start: String = ::std::iter::repeat('A').take(4096).collect();
            fs_.add(
                TestFile::new("/a")
                    .with_contents(format!("{}_a", start))
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs_.add(
                TestFile::new("/b")
                    .with_contents(format!("{}_b", start))
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b"].iter().map(PathBuf::from).collect();

        let mut fc = FileCataloger::new(fs);
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
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            let contents: String = ::std::iter::repeat('A').take(4096).collect();
            fs_.add(
                TestFile::new("/a")
                    .with_contents(contents.clone())
                    .with_metadata(TestMD::new())
                    .with_inode(1),
            );
            fs_.add(
                TestFile::new("/b")
                    .with_contents(contents)
                    .with_metadata(TestMD::new())
                    .with_inode(2),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b"].iter().map(PathBuf::from).collect();

        let mut fc = FileCataloger::new(fs);
        for file in &files {
            fc.insert(file);
        }

        let repeats = fc.get_repeats();
        assert_eq!(1, repeats.len());
        let dups = &repeats[0].0;
        assert_eq!(2, dups.len());
        assert!(dups.contains(&PathBuf::from("/a")));
        assert!(dups.contains(&PathBuf::from("/b")));
    }

    #[test]
    fn dup_test_hard_links() {
        // hard links to the same file should be flagged as duplicates
        // even if they (somehow) have different contents
        let fs = TestFileSystem::new();
        {
            let mut fs_ = fs.borrow_mut();
            fs_.create_dir("/");
            // note that all test files passed to FileCataloger must have metadata
            fs_.add(
                TestFile::new("/a")
                    .with_inode(1)
                    .with_contents(String::from("AAAA"))
                    .with_metadata(TestMD::new()),
            );
            fs_.add(
                TestFile::new("/b")
                    .with_inode(1)
                    .with_contents(String::from("BBBB"))
                    .with_metadata(TestMD::new()),
            );
        }
        let files: HashSet<_> = vec!["/a", "/b"].iter().map(PathBuf::from).collect();

        let mut fc = FileCataloger::new(fs);
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
