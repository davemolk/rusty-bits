use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use clap::Parser;
use arboard::Clipboard;

// according to https://archive.ph/faq, archive.is supports 
// newest and oldest as direct calls. this works in a browser,
// but i saw capchas returned for the cli, so default to
// timemap, which is explicitly mentioned in the memento protocol
// here: https://mementoweb.org/depot/native/archiveis/.
//
// we will just parse the full result and return what user asked for.
const ARCHIVE_TODAY: &str = "https://archive.is/timemap/";

#[derive(Parser)]
pub struct Args {
    /// url to search
    #[arg(short, long)]
    url: String,
    /// return the oldest archived (returns newest by deafult)
    #[arg(short, long)]
    oldest: bool,
    /// return all archived
    #[arg(short, long)]
    all: bool,
    /// print to stdout.
    /// default is to copy link to clipboard
    /// (newest in the case of multiple)
    #[arg(short, long)]
    print: bool,
    /// display date and url
    #[arg(short, long)]
    verbose: bool,
}

pub fn run(args: Args) -> Result<()> {
    let url = format!("{}{}", ARCHIVE_TODAY, args.url);
    let data = request(&url)?;
    let timemap = parse(&data);
    let result_url = match (args.oldest, args.all) {
        (true, _) => timemap.last.as_ref(),
        (_, true) => {
            // handle print here since we won't know where result_url comes
            // from below
            if args.print {
                // print newest one
                print_results(timemap.first.as_ref());
                for memento in &timemap.mementos {
                    let s = format!("{}: {}", memento.url, memento.datetime);
                    print_results(Some(&s));
                }
                print_results(timemap.last.as_ref());
                return Ok(());
            }
            // assign newest to copy below
            timemap.first.as_ref()
        },
        _ => timemap.first.as_ref(),
    };
    if args.print {
        print_results(result_url);
    } else {
        copy_results(result_url)?;
    }
    Ok(())
}

fn request(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url)
        .headers(default_headers())
        .send()?;
    Ok(resp.text()?)
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("github.com/davemolk/rusty-bits/no_paywall"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

fn print_results(result: Option<&String>) {
    if let Some(r) = result {
        println!("{}", r);
    } else {
        println!("no results");
    }
}

fn copy_results(result: Option<&String>) -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    if let Some(r) = result {
        clipboard.set_text(r)?;
        println!("link copied to clipboard")
    } else {
        println!("no results");
    }
    Ok(())
}

#[derive(Debug, Default)]
struct Timemap {
    original: Option<String>,
    timegate: Option<String>,
    first: Option<String>,
    last: Option<String>,
    mementos: Vec<Memento>,
}

#[derive(Debug)]
struct Memento {
    url: String,
    datetime: String,
}

fn parse(data: &str) -> Timemap {
    let mut tm = Timemap::default();
    if data.is_empty() {
        return tm;
    }
    let hits: Vec<&str> = data.split(",\n").collect();
    for hit in hits {
        let parts: Vec<&str> = hit.split("; ").collect();
        let url = parts[0].trim_matches(|c| c == '<' || c == '>').to_string();
        let (mut rel, mut datetime) = (None, None);
        for part in &parts {
            if let Some(val) = part.strip_prefix("rel=") {
                rel = Some(val.trim_matches('"').to_string());
            }
            if let Some(val) = part.strip_prefix("datetime=") {
                datetime = Some(val.trim_matches('"').to_string());
            }
        }
        match rel.as_deref() {
            Some("original") => tm.original = Some(url),
            Some("timegate") => tm.timegate = Some(url),
            Some(rel) if rel.contains("first") => tm.first = Some(url),
            Some(rel) if rel.contains("last") => tm.last = Some(url),
            Some("memento") => {
                if let Some(datetime) = datetime {
                    tm.mementos.push(Memento { url, datetime })
                }
            }
            _ => {}
        }
    }
    tm
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn parse_success() {
        let data = read_to_string("tests/data/res.txt").unwrap();
        let m = parse(&data);
        assert_eq!(m.mementos.len(), 2);
        assert!(m.first.is_some());
        assert_eq!(String::from("http://archive.md/20250128213048/https://www.denverpost.com/2025/01/28/ice-immigration-raids-aurora-denver-donald-trump/"), m.first.unwrap());
        assert!(m.last.is_some());
        assert_eq!(String::from("http://archive.md/20250130174844/https://www.denverpost.com/2025/01/28/ice-immigration-raids-aurora-denver-donald-trump/"), m.last.unwrap());
    }
}
