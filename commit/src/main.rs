use clap::Parser;
use commit::Args;

fn main() {
    commit::run(Args::parse());
}
