use cff::{Args, run};
use clap::Parser;
use std::process::exit;

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        exit(1);
    }
}

