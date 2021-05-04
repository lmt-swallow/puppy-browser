use log::Level;

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
