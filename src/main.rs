use log::Level;
use puppy::cli;
use structopt::StructOpt;

fn setup_logger(level: Level) -> Result<(), fern::InitError> {
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

fn main() {
    let opts: cli::Opts = cli::Opts::from_args();

    if let Some(level) = opts.verbose.log_level() {
        setup_logger(level).unwrap();
    }

    let exit_code = match opts.sub_command {
        cli::SubCommand::Open(sub_opts) => cli::subcommand::open::run(sub_opts),
        cli::SubCommand::Completion(sub_opts) => cli::subcommand::completion::run(sub_opts),
    };

    std::process::exit(exit_code)
}
