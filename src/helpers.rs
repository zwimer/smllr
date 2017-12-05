/// The number of bytes that will be read and hashed for a file
/// for the FirstKBytesProxy pass
pub const FIRST_K_BYTES: usize = 4096;

/// Uniquely identify a file by its device id and inode
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ID {
    /// Device ID (unique to Linux)
    pub dev: u64,
    /// Filesystem Inode
    pub inode: u64,
}

/// Represent the first K bytes of a file

pub fn prettify_bytes(b: u64) -> String {
    if b < 1024 {
        format!("{} B", b)
    } else if b < 1024 * 1024 {
        format!("{} KB", b / 1024)
    } else if b < 1024 * 1024 * 1024 {
        format!("{} MB", b / 1024 / 1024)
    } else if b < 1024 * 1024 * 1024 * 1024 {
        format!("{} GB", b / 1024 / 1024 / 1024)
    } else {
        // insert commas
        let mut s = format!("{} B", b);
        let mut i = s.len() as i64 - 2 - 3;
        while i > 0 {
            s.insert(i as usize, ',');
            i -= 3;
        }
        s
    }
}
