use super::subcommand::*;
use clap_verbosity_flag::Verbosity;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(flatten)]
    pub verbose: Verbosity,

    #[structopt(subcommand)]
    pub sub_command: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    Open(open::Opts),
    Completion(completion::Opts),
}
