use {
    std::{
        io,
    },
    fern,
    log,
};


pub fn setup_logging(verbosity: u64, logpath: Option<&str>) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();
    base_config = match verbosity {
        0 => base_config
            .level(log::LevelFilter::Error)
            .level_for("async_std::task::block_on", log::LevelFilter::Warn)
            .level_for("sled::meta", log::LevelFilter::Error)
            .level_for("sled::pagecache", log::LevelFilter::Error),
        1 => base_config
            .level(log::LevelFilter::Warn)
            .level_for("async_std::task::block_on", log::LevelFilter::Warn)
            .level_for("sled::pagecache", log::LevelFilter::Warn),
        2 => base_config
            .level(log::LevelFilter::Info)
            .level_for("async_std::task::block_on", log::LevelFilter::Warn)
            .level_for("mio::poll", log::LevelFilter::Warn)
            .level_for("sled::meta", log::LevelFilter::Info)
            .level_for("sled::pagecache", log::LevelFilter::Info),
        3 => base_config
            .level(log::LevelFilter::Debug)
            .level_for("sled::pagecache", log::LevelFilter::Info)
            .level_for("sled::meta", log::LevelFilter::Info)
            .level_for("mio::poll", log::LevelFilter::Info),
        _4_or_more => base_config
            .level(log::LevelFilter::Trace)
            .level_for("mio::poll", log::LevelFilter::Error)
            .level_for("sled::pagecache", log::LevelFilter::Error)
            .level_for("sled::meta", log::LevelFilter::Error)
            .level_for("sled::tree", log::LevelFilter::Error)
            .level_for("async_std::task::block_on", log::LevelFilter::Error),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file(logpath.unwrap_or("copernica.log"))?);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            // special format for debug messages coming from our own crate.
            if record.level() > log::LevelFilter::Info && record.target() == "cmd_program" {
                out.finish(format_args!(
                    "---\nDEBUG: {}: {}\n---",
                    chrono::Local::now().format("%H:%M:%S"),
                    message
                ))
            } else {
                out.finish(format_args!(
                    "[{}][{}] {}",
                    record.target(),
                    record.level(),
                    message
                ))
            }
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;

    Ok(())
}
