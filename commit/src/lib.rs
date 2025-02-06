use std::fmt::Display;
use arboard::Clipboard;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum CommitType {
    Fix,
    Feat,
    Build,
    Chore,
    Ci,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
}

impl Display for CommitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{}", 
            match self {
                CommitType::Fix => "fix",
                CommitType::Feat => "feat",
                CommitType::Build => "build",
                CommitType::Chore => "chore",
                CommitType::Ci => "ci",
                CommitType::Docs => "docs",
                CommitType::Style => "style",
                CommitType::Refactor => "refactor",
                CommitType::Perf => "perf",
                CommitType::Test => "test",
            }
        )
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long("type"))]
    type_commit: CommitType,
    
    #[arg(short, long)]
    scope: Option<String>,
    
    #[arg(short, long)]
    description: String,
    
    #[arg(short, long)]
    breaking: bool,
    
    #[arg(long)]
    body: Option<String>,

    #[arg(short, long)]
    footer: Option<Vec<String>>,
    /// print msg to terminal instead of copying to clipboard
    #[arg(long)]
    pub dry: bool,
}

pub fn run(args: Args) {
    let dry = args.dry;
    let msg = format_msg(args);
    if !dry {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(&msg).unwrap();
        println!("commit copied to clipboard");
    } else {
        println!("{msg}")
    }
}

fn format_msg(args: Args) -> String {
    let exc = if args.breaking { "!" } else { "" };
    let scope = args.scope.map_or_else(String::new, |s| format!("({s})"));
    let body = args.body.map_or_else(String::new, |b| format!("\n\n{b}")); 
    let footer = args.footer.map_or_else(String::new, |f| format!("\n\n{}", f.join("\n")));
    format!("{}{}{}: {}{}{}", args.type_commit, scope, exc, args.description, body, footer)
}