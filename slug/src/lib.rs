use anyhow::{anyhow, Result};
use clap::Parser;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::fs;
use std::str::FromStr;
use log::{
    debug, error, info, warn
};

#[derive(Parser, Debug)]
pub struct Args {
    /// path to start slugging
    // #[clap(short, long)]
    #[clap(required=true)]
    path: PathBuf,

    /// crawl directories recursively and sluggify
    #[clap(short, long)]
    crawl: bool,

    /// sluggify directories too
    #[clap(short, long)]
    dirs: bool,

    /// slugginate (defaults to dry run)
    #[clap(long)]
    slug: bool,

    /// maximum slugginate, convert letters
    /// to lowercase and remove anything not
    /// alphanumeric plus . and -
    /// 
    /// default behavior is to convert spaces to dashes (-)
    #[clap(short, long)]
    all: bool,

    /// slugginate hidden files and dirs
    /// (assuming dirs bool is true)
    #[clap(long)]
    hidden: bool,

    /// ignore naming conflicts
    /// (default behavior is to error)
    #[clap(long)]
    ignore: bool,

    /// silent mode
    #[clap(long)]
    pub silent: bool,

    /// debug
    #[clap(long)]
    pub debug: bool,

    /// use a custom separator
    /// to replace spaces
    #[clap(short, long)]
    separator: Option<char>,
}

pub struct Slug {
    crawl: bool,
    include_dirs: bool,
    slug: bool,
    max_slug: bool,
    include_hidden: bool,
    ignore_conflicts: bool,
    custom_separator: Option<char>,
}

impl Slug {
    pub fn new(args: Args) -> Self {
        Slug {
            crawl: args.crawl,
            include_dirs: args.dirs,
            slug: args.slug,
            max_slug: args.all,
            include_hidden: args.hidden,
            ignore_conflicts: args.ignore,
            custom_separator: args.separator
        }
    }
    fn crawl_dir(&self, path: &Path) -> Result<()> {
        if path.is_file() {
            info!("renaming target file");
            self.rename_entry(path)?;
        } else if path.is_dir() {
            info!("scanning dir: {:?}", path.as_os_str());
            for entry in fs::read_dir(path).map_err(|e| anyhow!("error reading dir {:?}: {}", path, e))? {
                let entry = entry.map_err(|e| anyhow!("error processing entry: {}", e))?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    // descend before renaming
                    if self.crawl {
                        self.crawl_dir(&entry_path)?;
                    }
                } else if entry_path.is_file() {
                    debug!("analyzing file: {:?}", entry_path);
                    self.rename_entry(&entry_path)?;
                } else {
                    debug!("skipping {:?}", entry_path);
                }
            }
            // rename the dir i start with once everything inside is handled
            if self.include_dirs {
                self.rename_entry(&path)?;
            }
            
        } else {
            return Err(anyhow!("unable to handle {:?}", path));
        }
        Ok(())
    }
    fn rename_entry(&self, path: &Path) -> Result<()> {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(".") && !self.include_hidden {
                debug!("ignoring hidden entity: {}", name);
                return Ok(());
            }
            let slug = if self.max_slug { 
                slugginate(name, self.custom_separator) 
            } else { 
                simple_slug(name, self.custom_separator) 
            };
            // nothing changed, nothing to do,
            if slug == name {
                return Ok(());
            }
            // file exists, don't clobber
            let new_path = path.with_file_name(&slug);
            if new_path.exists() {
                if self.ignore_conflicts {
                    warn!("name conflict detected for {}, ignoring", slug);
                    return Ok(());
                } else {
                    return Err(anyhow!("slugginated file conflicts with existing file: {}", slug));
                }
            }
            // actually rewrite
            if self.slug {
                let new_path = path.with_file_name(&slug);
                fs::rename(path, new_path)?;
            }                
            info!("SLUGGED: {} -> {}", name, &slug);
        }
        Ok(())
    }
}

pub fn run(args: Args) -> Result<()> {
    let path = args.path.clone();
    if !args.slug && args.all {
        return Err(anyhow!("can't have 'all' without 'slug'"))
    }
    if !args.slug {
        info!("####################");
        info!("### dry slug run ###");
        info!("####################");
        info!("\n");
    }
    let slug = Slug::new(args);
    slug.crawl_dir(&path)?;
    Ok(())
}

fn simple_slug(input: &str, separator: Option<char>) -> String {
    let sep = separator.unwrap_or('-');
    let mut slugged = String::new();
    let mut in_sequence = false;
    for c in input.trim().chars() {
        if c == ' ' {
            if !in_sequence {
                slugged.push(sep);
                in_sequence = true;
            }
        } else {
            slugged.push(c);

            in_sequence = false;
        }
    }
    slugged
}

fn slugginate(input: &str, separator: Option<char>) -> String {
    let sep = separator.unwrap_or('-');
    let mut slugged = String::new();
    let mut in_sequence = false;
    
    let spaces_not_treated = input.to_ascii_lowercase().trim()
        .chars()
        .filter(|&c| c.is_alphanumeric() || c == '.' || c == sep || c == ' ')
        .collect::<String>();
    for c in spaces_not_treated.chars() {
        if c == ' ' {
            if !in_sequence {
                slugged.push(sep);
                in_sequence = true
            }
        } else {
            slugged.push(c);
            in_sequence = false;
        }
    }
    slugged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_slug() {
        assert_eq!(simple_slug("Afile.txt", None), "Afile.txt", "leaves capitals alone");
        assert_eq!(simple_slug("Afile1.txt", None), "Afile1.txt", "leaves numbers alone");
        assert_eq!(simple_slug("A file.txt", None), "A-file.txt", "replace space with dash");
        assert_eq!(simple_slug("A   file.txt", None), "A-file.txt", "replace multiple spaces with single dash");
        assert_eq!(simple_slug(" a file.txt", None), "a-file.txt", "trim whitespace");
        assert_eq!(simple_slug(" a file.txt", Some('_')), "a_file.txt", "custom separator");
    }

    #[test]
    fn test_slugginate() {
        assert_eq!(slugginate("Afile", None), "afile", "capital -> lowercase");
        assert_eq!(slugginate("file.txt", None), "file.txt", "leaves periods alone");
        assert_eq!(slugginate("a-file.txt", None), "a-file.txt", "leaves dashes alone");
        assert_eq!(slugginate("  a-file.txt  ", None), "a-file.txt", "trims whitespace");
        assert_eq!(slugginate("file1.txt", None), "file1.txt", "leaves numbers alone");
        assert_eq!(slugginate("A file.txt", None), "a-file.txt", "replace space with dash");
        assert_eq!(slugginate("A   file.txt", None), "a-file.txt", "replace multiple spaces with single dash");
        assert_eq!(slugginate("+=!@#$%^&*()_\\|'\";:<>,?/{}[]`~±§a", None), "a", "drops special characters");
        assert_eq!(slugginate("here    is a file.txt  ", Some('_')), "here_is_a_file.txt", "custom separator");
    }
}