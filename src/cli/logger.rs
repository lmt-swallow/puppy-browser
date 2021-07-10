//! This module defines loggers for `puppy` command.

use log::Level;

/// `setup_logger` initializes a logger with the given log level for CLI.
pub fn setup_logger(level: Level) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level.to_level_filter())
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
