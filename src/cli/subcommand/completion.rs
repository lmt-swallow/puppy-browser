use std::io;
use structopt::clap::Shell;
use structopt::StructOpt;

use crate::cli::CommonOpts;

#[derive(StructOpt, Debug)]
pub enum Opts {
    Zsh,
    Bash,
    Fish,
}

pub fn run(_common_opts: CommonOpts, opts: Opts) -> i32 {
    match opts {
        Opts::Bash => completion(Shell::Bash),
        Opts::Zsh => completion(Shell::Zsh),
        Opts::Fish => completion(Shell::Fish),
    };

    return 0;
}

fn completion(s: Shell) {
    super::super::Opts::clap().gen_completions_to(env!("CARGO_PKG_NAME"), s, &mut io::stdout())
}
