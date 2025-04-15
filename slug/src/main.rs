use clap::Parser;
use slug::{Args, run};
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let args = Args::parse();
    let mut builder = Builder::new();
    let level = if args.silent {
        LevelFilter::Off
    } else if args.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::max()
    };
    builder.filter_level(level);
    builder.init();

    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
