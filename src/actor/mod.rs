
use vfs::{self, VFS, File};
use catalog::proxy::Duplicates;

mod selector;
use self::selector::{Selector};


trait FileActor<V: VFS> {
    fn act(vfs: &V, dups: Duplicates);
}

