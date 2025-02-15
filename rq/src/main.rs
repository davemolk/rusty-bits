use clap::Parser;
use rq::Args;

fn main() {
    let args = Args::parse();
    if let Err(e) = rq::run(args) {
        eprintln!("error: {e}", );
        std::process::exit(1);
    }
}
