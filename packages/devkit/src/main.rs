use std::io::stdout;
use std::process::ExitCode;

use clap::Parser;
use stellar_devkit::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let code = cli.run(&mut stdout().lock());
    ExitCode::from(code as u8)
}
