use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: String,
}

pub fn run(opts: Opts) -> i32 {
    log::debug!("open subcommand called with {}", opts.url);
    todo!();
}
