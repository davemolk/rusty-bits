use clap::Parser;
use pw::{PasswordGenerator, Args};

fn main() {
    let args = Args::parse();
    let mut pw = PasswordGenerator::new(args);
    if let Err(e) = pw.run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
