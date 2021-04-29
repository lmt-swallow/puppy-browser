use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub input: String,
}

pub fn run(build_opts: Opts) -> i32 {
    log::debug!("build subcommand called with {}", build_opts.input);
    todo!();
}
