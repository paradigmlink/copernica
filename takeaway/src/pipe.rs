use {
    anyhow::Result,
    nix::{
        unistd,
        sys::stat::Mode,
        errno::Errno,
    },
    std::{
        fs::{OpenOptions, File},
        path::Path,
        io::{self, Write, Read, Seek, SeekFrom},
    },
};

pub struct Pipe {
	file: File
}

impl Pipe {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Pipe> {
		let res = unistd::mkfifo(path.as_ref(), Mode::S_IRWXU);
		if res.is_err() {
			match res.err().unwrap() {
				nix::Error::Sys(e) => {
					if e != Errno::EEXIST {
						res?
					}
				}
				_ => res?
			}
		}
		let file = OpenOptions::new()
			.create(true)
			.read(true)
			.write(true)
			.open(path.as_ref())?;
		Ok(Pipe { file })
	}

	pub fn file(&mut self) -> &mut File {
		&mut self.file
	}
}
