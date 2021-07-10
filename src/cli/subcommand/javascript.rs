//! This module defines `javascript` subcommand.

use crate::{
    cli::{logger, CommonOpts},
    javascript::JavaScriptRuntime,
};
use std::io::{self, Write};

/// `run` launches an JS interpreter with V8.
pub fn run(common_opts: CommonOpts) -> i32 {
    if let Some(level) = common_opts.verbose.log_level() {
        logger::setup_logger(level).unwrap();
    }

    let mut runtime = JavaScriptRuntime::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    println!();
                    return 0;
                }

                match runtime.execute("(shell)", &buf) {
                    Ok(message) => println!("{}", message),
                    Err(e) => eprintln!("{}", e),
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
