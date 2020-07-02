use {
    fuse::{
        FileAttr, FileType, Filesystem, ReplyAttr, ReplyCreate,
        ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
        ReplyStatfs, ReplyWrite, Request,
    },
    libc::{ENOENT, ENOTDIR, ENOTRECOVERABLE, EREMOTE},
    super::{Config, File, FileId, FileManager, CopernicaFacade},
    anyhow::{Result},
    time::Timespec,
    std::{
        ffi::OsStr,
    },
};

pub type Inode = u64;

pub struct NullFs;
impl Filesystem for NullFs {}

pub struct CopernicaFs {
    manager: FileManager,
}

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1 second

impl CopernicaFs {
    pub fn with_config(config: Config) -> Result<Self> {
        Ok(CopernicaFs {
            manager: FileManager::with_copernica_facade(
                CopernicaFacade::new(&config),
            )?,
        })
    }
}

impl Filesystem for CopernicaFs {
    fn lookup(&mut self, _req: &Request, parent: Inode, name: &OsStr, reply: ReplyEntry) {
        // self.manager.sync();

        let name = name.to_str().unwrap().to_string();
        let id = FileId::ParentAndName { parent, name };

        match self.manager.get_file(&id) {
            Some(ref file) => {
                reply.entry(&TTL, &file.attr, 0);
            }
            None => {
                reply.error(ENOENT);
            }
        };
    }

    fn getattr(&mut self, _req: &Request, ino: Inode, reply: ReplyAttr) {
        // self.manager.sync();
        match self.manager.get_file(&FileId::Inode(ino)) {
            Some(file) => {
                reply.attr(&TTL, &file.attr);
            }
            None => {
                reply.error(ENOENT);
            }
        };
    }
}
