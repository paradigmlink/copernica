use {
    fuse::{FileAttr, FileType},
    id_tree::NodeId,
};

type Inode = u64;


#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub attr: FileAttr,
    //pub copernica_file: Option<>,
}

#[derive(Debug, Clone)]
pub enum FileId {
    Inode(Inode),
    CopernicaId(String),
    NodeId(NodeId),
    ParentAndName { parent: Inode, name: String },
}

impl File {

    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn inode(&self) -> Inode {
        self.attr.ino
    }

}
