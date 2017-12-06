//! Hash files (in whole or in part) generically so user can determine algorithm at runtime

use md5;
use tiny_keccak;

use std::fmt::Debug;
use std::hash;

// The standard library has a similar trait: std::hash::Hash
// However, Rust does not allow you to implement a trait from an external crate
//  on an object in an external crate; one of them must be your own
// So to implement this we use our own FileHash trait

/// Easy way to hash byte arrays
pub trait FileHash: Debug {
    // Output must be Debug for the whole type to be
    // must be Eq and Hash to be a key for a hash table
    // must be Clone because hashes sometimes must be stored redundantly
    /// Output type of hashing (different algorithms returns differently sized outputs)
    type Output: Debug + Clone + Eq + hash::Hash; //+ ::std::ops::Index<usize>;
    /// Hash an array of bytes and return the result
    fn hash(bytes: &[u8]) -> Self::Output;
}

/// Generate 128-bit MD5 digest
#[derive(Debug)]
pub struct Md5Sum;

/// Generate 256-bit Sha3 digest
#[derive(Debug)]
pub struct Sha3Sum;

// Md5Sum implementation wraps around `md5` crate
// returns a 128-bit hash
impl FileHash for Md5Sum {
    type Output = [u8; 16];

    fn hash(bytes: &[u8]) -> Self::Output {
        *md5::compute(bytes)
    }
}

// Sha3Sum implementation wraps around `tiny_keccak` crate
// returns a 256-bit hash
impl FileHash for Sha3Sum {
    type Output = [u8; 32];

    fn hash(bytes: &[u8]) -> Self::Output {
        let mut sha = tiny_keccak::Keccak::new_sha3_256();
        let mut arr = [0u8; 32];
        sha.update(bytes);
        sha.finalize(&mut arr);
        arr
    }
}
