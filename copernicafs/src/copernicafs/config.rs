use std::path::{PathBuf};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub debug: Option<bool>,
    pub mount_check: Option<bool>,
    pub mount_options: Option<Vec<String>>,
    pub config_dir: Option<PathBuf>,
}

impl Config {
    pub fn debug(&self) -> bool {
        self.debug.unwrap_or(false)
    }

    pub fn mount_check(&self) -> bool {
        self.mount_check.unwrap_or(true)
    }

    pub fn mount_options(&self) -> Vec<String> {
        match self.mount_options {
            Some(ref options) => options.clone(),
            None => Vec::new(),
        }
    }

    pub fn config_dir(&self) -> &PathBuf {
        self.config_dir.as_ref().unwrap()
    }
}
