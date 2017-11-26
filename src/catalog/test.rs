#[cfg(test)]
mod test {

    use vfs;
    use vfs::{TestFileSystem, TestFile, TestMD};

    use catalog::FileCataloger;

    use std::rc::Rc;
    use std::path::PathBuf;
    use std::collections::HashSet;

    #[test]
    fn catalog_test() {
        panic!("catalog test");
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
