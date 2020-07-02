use {
    super::{File, FileId, CopernicaFacade},
    id_tree::{Node, NodeId, Tree, TreeBuilder},
    std::{
        collections::HashMap,
    },
    anyhow::{Context, Result, Error},
};
pub type Inode = u64;
pub type CopernicaId = String;

pub struct FileManager {
    tree: Tree<Inode>,
    pub files: HashMap<Inode, File>,
    pub node_ids: HashMap<Inode, NodeId>,
    pub copernica_ids: HashMap<CopernicaId, Inode>,

    pub cf: CopernicaFacade,

}

impl FileManager {
    pub fn with_copernica_facade(
        cf: CopernicaFacade,
    ) -> Result<Self> {
        let mut manager = FileManager {
            tree: TreeBuilder::new().with_node_capacity(500).build(),
            files: HashMap::new(),
            node_ids: HashMap::new(),
            copernica_ids: HashMap::new(),
            cf,
        };
        Ok(manager)
    }

    pub fn get_file(&self, id: &FileId) -> Option<&File> {
        let inode = self.get_inode(id)?;
        self.files.get(&inode)
    }

    pub fn get_inode(&self, id: &FileId) -> Option<Inode> {
        match id {
            FileId::Inode(inode) => Some(*inode),
            FileId::CopernicaId(copernica_id) => self.copernica_ids.get(copernica_id).cloned(),
            FileId::NodeId(node_id) => self
                .tree
                .get(&node_id)
                .map(|node| node.data())
                .ok()
                .cloned(),
            FileId::ParentAndName {
                ref parent,
                ref name,
            } => self
                .get_children(&FileId::Inode(*parent))?
                .into_iter()
                .find(|child| child.name() == *name)
                .map(|child| child.inode()),
        }
    }

    pub fn get_children(&self, id: &FileId) -> Option<Vec<&File>> {
        let node_id = self.get_node_id(&id)?;
        let children: Vec<&File> = self
            .tree
            .children(&node_id)
            .unwrap()
            .map(|child| self.get_file(&FileId::Inode(*child.data())))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();

        Some(children)
    }

    pub fn get_node_id(&self, file_id: &FileId) -> Option<NodeId> {
        match file_id {
            FileId::Inode(inode) => self.node_ids.get(&inode).cloned(),
            FileId::CopernicaId(copernica_id) => self.get_node_id(&FileId::Inode(
                self.get_inode(&FileId::CopernicaId(copernica_id.to_string()))
                    .unwrap(),
            )),
            FileId::NodeId(node_id) => Some(node_id.clone()),
            ref pn @ FileId::ParentAndName { .. } => {
                let inode = self.get_inode(&pn)?;
                self.get_node_id(&FileId::Inode(inode))
            }
        }
    }

}
