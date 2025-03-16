use anyhow::{anyhow, Result};
use clap::Parser;
use std::{
    fmt::{Display, Formatter}, fs, path::{Path, PathBuf}, str::FromStr, time::Duration
};

#[derive(Debug, Clone)]
pub enum Source {
    Short,
    Medium,
    Large,
    Custom(PathBuf),
}

#[derive(Parser, Debug)]
pub struct Args {
    /// wordlist source to use 
    #[clap(short, long, default_value_t=Source::Short)]
    source: Source,

    /// print to stdout
    #[clap(short, long)]
    print: bool,

    /// number of words to include (default 6)
    #[clap(short, long="number", default_value_t=6)]
    num_words: u8,

    /// word separator (default -)
    #[clap(short='b', long="between", default_value_t='-')]
    separator: char,
}

impl FromStr for Source {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Source::Short);
        }
        match s.to_lowercase().as_str() {
            "short" => Ok(Source::Short),
            "medium" => Ok(Source::Medium),
            "large" => Ok(Source::Large),
            _ => {
                let path = PathBuf::from(s);
                if path.exists() {
                    Ok(Source::Custom(path))
                } else {
                    Err("Invalid source or path".to_string())
                }
            }
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Source::Short => "short",
                Source::Medium => "medium",
                Source::Large => "large",
                Source::Custom(_) => "custom",
            }
        )
    }
}

#[derive(Debug)]
pub struct PasswordGenerator {
    args: Args,
    client: reqwest::blocking::Client,
    path: PathBuf,
}

impl PasswordGenerator {
    const LARGE_LIST_SOURCE: &str = "https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt";
    const MEDIUM_LIST_SOURCE: &str = "https://www.eff.org/files/2016/09/08/eff_short_wordlist_2_0.txt";
    const SHORT_LIST_SOURCE: &str = "https://www.eff.org/files/2016/09/08/eff_short_wordlist_1.txt";    
    const SHORT_LIST: &str = "short.txt";
    const MEDIUM_LIST: &str = "short1.txt";
    const LARGE_LIST: &str = "large.txt";

    pub fn new(args: Args) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build().expect("failed to create client");
        let mut path: PathBuf = dirs::home_dir().expect("no home directory found");
        path.push(".pw");
        // need to confirm it exists
        fs::create_dir_all(&path).expect("failed to create directory for pw");
        PasswordGenerator{ 
            args, 
            client,
            path,
        }
    }

    fn get_data(&self, path: &str, url: &str) -> Result<()> {
        // check if it's there, exit if yes, get it and clean if not
        let mut resource_path: PathBuf = self.path.clone();
        resource_path.push(path);
        if resource_path.exists() {
            return Ok(())
        }
        let res = self.client.get(url)
            .send()?;
        if !res.status().is_success() {
            return Err(anyhow!("unexpected status: {:?}", res.status().canonical_reason()));
        }
        // todo, need to clean the text
        fs::write(resource_path, res.text()?)?;
        Ok(())
    }
 
    pub fn run(&mut self) -> Result<()> {
        // if not custom, check if we have existing file, download if not
        match &self.args.source {
            Source::Short => self.get_data(Self::SHORT_LIST, Self::SHORT_LIST_SOURCE)?,
            Source::Medium => self.get_data(Self::MEDIUM_LIST, Self::MEDIUM_LIST_SOURCE)?,
            Source::Large => self.get_data(Self::LARGE_LIST, Self::LARGE_LIST_SOURCE)?,
            Source::Custom(c) => {
                // todo
            },
        }
        Ok(())
    }
}


