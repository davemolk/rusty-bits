use serde::Deserialize;
use serde_json;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Default, Debug, Deserialize)]
struct Calls {
    #[serde(rename = "Five Calls")]
    five_calls: HashMap<String, Vec<String>>,
}

pub fn run() -> Result<()> {
    let path = "~/.calls/calls.json";
    let expanded_path = shellexpand::tilde(path).to_string();
    let file = File::open(expanded_path)
        .with_context(|| format!("Failed to open file from {}", path))?;
    let reader = BufReader::new(file);
    let calls: Calls = serde_json::from_reader(reader)?;
    for (name, numbers) in &calls.five_calls {
        println!("{}:", name);
        for num in numbers {
            println!("{}", num);
        }
        println!();
    }
    Ok(())
}