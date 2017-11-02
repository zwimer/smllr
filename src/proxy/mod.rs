
use std::collections::HashMap;
use std::rc::Rc;
use std::path::PathBuf;

type Hash = [u8;32];

#[derive(Debug)]
struct FirstKBytesProxy {
    //delay: Option<&HashProxy>,
    delay: Rc<Vec<PathBuf>>,
    thunk: HashMap<Hash, HashProxy>,
}

#[derive(Debug)]
struct HashProxy;

