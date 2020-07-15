#[macro_use]
extern crate clap;
extern crate config;
extern crate ctrlc;
extern crate itertools;
extern crate fuse;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;
extern crate xdg;

use {
    std::{
        fs,
        io::prelude::*,
        iter,
        ffi::OsStr,
        thread,
        time,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    },
    itertools::Itertools,
    clap::App,
    copernicafs::{Config, NullFs, CopernicaFs},
    anyhow::{Context, Result},
};

const DEBUG_LOG: &str = "";

const INFO_LOG: &str =
    "fuse::session=error,info";

const DEFAULT_CONFIG: &str = r#"
### This is the default configuration file that Copernica Filesystem (copernicafs) uses.
### It should be placed in $XDG_CONFIG_HOME/copernicafs/copernicafs.toml, which is usually
### defined as $HOME/.config/copernicafs/copernicafs.toml
# Show additional logging info?
debug = false
# Perform a mount check and fail early if it fails. Disable this if you
# encounter this error:
#
#     fuse: attempt to remount on active mount point: [...]
#     Could not mount to [...]: Undefined error: 0 (os error 0)
mount_check = true
"#;

fn mount_copernicafs(config: Config, mountpoint: &str) {
    let vals = config.mount_options();
    let mut options = iter::repeat("-o")
        .interleave_shortest(vals.iter().map(String::as_ref))
        .map(OsStr::new)
        .collect::<Vec<_>>();
    options.pop();

    if config.mount_check() {
        unsafe {
            match fuse::spawn_mount(NullFs {}, &mountpoint, &options) {
                Ok(session) => {
                    debug!("Test mount of NullFs successful. Will mount GCSF next.");
                    drop(session);
                }
                Err(e) => {
                    error!("Could not mount to {}: {}", &mountpoint, e);
                    return;
                }
            };
        }
    }

    info!("Creating and populating file system...");
    let fs: CopernicaFs = match CopernicaFs::with_config(config) {
        Ok(fs) => fs,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    info!("File system created.");

    unsafe {
        info!("Mounting to {}", &mountpoint);
        match fuse::spawn_mount(fs, &mountpoint, &options) {
            Ok(_session) => {
                info!("Mounted to {}", &mountpoint);

                let running = Arc::new(AtomicBool::new(true));
                let r = running.clone();

                ctrlc::set_handler(move || {
                    info!("Ctrl-C detected");
                    r.store(false, Ordering::SeqCst);
                })
                .expect("Error setting Ctrl-C handler");

                while running.load(Ordering::SeqCst) {
                    thread::sleep(time::Duration::from_millis(50));
                }
            }
            Err(e) => error!("Could not mount to {}: {}", &mountpoint, e),
        };
    }
}


fn load_conf() -> Result<Config> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("copernicafs").unwrap();
    let config_file = xdg_dirs
        .place_config_file("copernicafs.toml")
        .with_context(|| format!("Cannot create configuration directory"))?;

    info!("Config file: {:?}", &config_file);

    if !config_file.exists() {
        let mut config_file = fs::File::create(config_file.clone())
            .with_context(|| format!("Could not create config file"))?;
        config_file.write_all(DEFAULT_CONFIG.as_bytes())?;
    }

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(config_file.to_str().unwrap()))
        .expect("Invalid configuration file");

    let mut config = settings.try_into::<Config>()?;
    config.config_dir = Some(xdg_dirs.get_config_home());

    Ok(config)
}

fn main() {
    let config = load_conf().expect("Could not load configuration file.");

    pretty_env_logger::formatted_builder()
        .parse_filters(if config.debug() { DEBUG_LOG } else { INFO_LOG })
        .init();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("mount") {
        let mountpoint = matches.value_of("mountpoint").unwrap();
        mount_copernicafs(config, mountpoint);
    }
}
