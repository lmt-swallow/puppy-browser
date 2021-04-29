use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub input: String,
}

pub fn run(simulate_opts: Opts) -> i32 {
    log::debug!("simulation subcommand called with {}", simulate_opts.input);
    todo!();
}
