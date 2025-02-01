use clap::Parser;
use commit::Args;
use arboard::Clipboard;

fn main() {
    let args = Args::parse();
    let dry = args.dry;
    let msg = commit::format_msg(args);
    if !dry {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(&msg).unwrap();
        println!("commit copied to clipboard")
    } else {
        println!("{msg}");
    }
}
