use archive::{Args, run};
use clap::Parser;

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        std::process::exit(1)
    }
}
