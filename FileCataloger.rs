


use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::Metadata;

mod FirstKBytesProxy;
mod IdToDuplicates;
mod ID;


struct FileCataloger{
    // member variables here
    file_catalog: Hashmap<u64, FirstKBytesProxy>,
    id_to_list: IdToDuplicates,
    lists_of_duplicates: HashSet<Duplicates>
}

impl FileCataloger{
    //functions here
    //form:
    //fn name(&self, args...) -> [ (), ]&ret] {...}

    fn New(){
        file_catalog = HashMap::New();
        id_to_list = IdToDuplicates::New();
        lists_of_duplicates = lists_of_duplicates::New();
    }

    fn add_file(&self, path : &PathBuf ) -> (){
        let md =fs::metadata(path);
        let id = ID::New(md.st_ino(), md.st_dev());

        if id_to_list.exists(id){
            id_to_list.get(id)->insert(path);
        } else{
            let retDup = file_catalog.insert(md.len(), path);
            id_to_list.insert(id, retDup);
            if retDup->len() == 2 {
                lists_of_duplicates.insert(retDup);
            }
        }
    }

    fn get_duplicate_set_iterator() -> &'a Duplicates {
        return lists_of_duplicates.iter();
        //free file_catalog
        //free id_to_list
    }

}
