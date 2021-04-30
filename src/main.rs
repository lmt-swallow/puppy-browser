use puppy::cli;
use structopt::StructOpt;

fn main() {
    let opts: cli::Opts = cli::Opts::from_args();

    let exit_code = match opts.sub_command {
        cli::SubCommand::Open(sub_opts) => cli::subcommand::open::run(opts.common_opts, sub_opts),
        cli::SubCommand::Completion(sub_opts) => {
            cli::subcommand::completion::run(opts.common_opts, sub_opts)
        }
    };

    std::process::exit(exit_code)
}
