use crate::{cli::CommonOpts, js::JavaScriptRuntime};
use std::io::{self, Write};

pub fn run(_common_opts: CommonOpts) -> i32 {
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
