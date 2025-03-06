use clap::Parser;
use anyhow::{anyhow, Result};
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::collections::HashMap;


#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum Format {
    Json,
    Yaml,
    Toml,
}

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    /// path to an optional source file.
    /// if no path is provided, will read from stdin.
    #[clap(short, long)]
    source_path: Option<PathBuf>,

    /// optional path to write conversion results.
    /// if no path is provided, will output to stdout.
    #[clap(short, long)]
    dest_path: Option<PathBuf>,

    /// file conversion to do:
    ///     JY: json to yaml
    ///     YJ: yaml to json
    ///     JT: json to toml
    ///     TJ: toml to json
    ///     YT: yaml to toml
    ///     TY: toml to yaml
    #[clap(required=true, value_parser=parse_conversion)]
    conversion: (Format, Format),
}

fn parse_conversion(conversion: &str) -> Result<(Format, Format)> {
    let c = match conversion.to_uppercase().as_str() {
        "JY" => (Format::Json, Format::Yaml),
        "JT" => (Format::Json, Format::Toml),
        "YJ" => (Format::Yaml, Format::Json),
        "YT" => (Format::Yaml, Format::Toml),
        "TJ" => (Format::Toml, Format::Json),
        "TY" => (Format::Toml, Format::Yaml),
        _ => {
            return Err(anyhow!("conversion unsupported"));
        }
    };
    Ok(c)
}

fn yaml_to_json(data: &str) -> Result<String> {
    let yaml_data: serde_yaml::Value = serde_yaml::from_str(data)?;
    Ok(serde_json::to_string_pretty(&yaml_data)?)
}

fn yaml_to_toml(data: &str) -> Result<String> {
    let yaml_data: serde_yaml::Value = serde_yaml::from_str(data)?;
    Ok(toml::to_string(&yaml_data)?)
}

fn json_to_yaml(data: &str) -> Result<String> {
    let json_data: serde_json::Value = serde_json::from_str(data)?;
    Ok(serde_yaml::to_string(&json_data)?)
}

fn json_to_toml(data: &str) -> Result<String> {
    let json_data: serde_json::Value = serde_json::from_str(data)?;
    Ok(toml::to_string(&json_data)?)
}

fn toml_to_json(data: &str) -> Result<String> {
    let toml_data: toml::Value = toml::de::from_str(data)?;
    Ok(serde_json::to_string_pretty(&toml_data)?)
}

fn toml_to_yaml(data: &str) -> Result<String> {
    let toml_data: toml::Value = toml::de::from_str(data)?;
    Ok(serde_yaml::to_string(&toml_data)?)
}

type ConversionFn = fn(&str) -> Result<String>;
type FormatPair = (Format, Format);

pub fn run(args: Args) -> Result<()> {
    let mut conversions: HashMap<FormatPair, ConversionFn> = HashMap::new();
    conversions.insert((Format::Yaml, Format::Json), yaml_to_json); 
    conversions.insert((Format::Yaml, Format::Toml), yaml_to_toml);
    conversions.insert((Format::Json, Format::Yaml), json_to_yaml);
    conversions.insert((Format::Json, Format::Toml), json_to_toml);
    conversions.insert((Format::Toml, Format::Json), toml_to_json);
    conversions.insert((Format::Toml, Format::Yaml), toml_to_yaml);

    let conversion_fn = conversions.get(&args.conversion).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Conversion function not found")
    })?;

    let data = if let Some(path) = args.source_path {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut input = String::new();
        reader.read_to_string(&mut input)?;
        input
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    };

    let converted_data = conversion_fn(&data)?;
    if let Some(path) = args.dest_path {
        write_data(&path, &converted_data)?;
    } else {
        println!("{}", &converted_data);
    }

    Ok(())
}

fn write_data(path: impl AsRef<Path>, data: &str) -> Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn compare_json_str(expected: &str, actual: &str) {
        let expected_json: serde_json::Value = serde_json::from_str(expected).unwrap();
        let actual_json: serde_json::Value = serde_json::from_str(actual).unwrap();
        assert_eq!(expected_json, actual_json)
    }

    fn compare_yaml_str(expected: &str, actual: &str) {
        let expected_yaml: serde_yaml::Value = serde_yaml::from_str(expected).unwrap();
        let actual_yaml: serde_yaml::Value = serde_yaml::from_str(actual).unwrap();
        assert_eq!(expected_yaml, actual_yaml)
    }

    fn compare_toml_str(expected: &str, actual: &str) {
        let expected_toml: toml::Value = toml::de::from_str(expected).unwrap();
        let actual_toml: toml::Value = toml::de::from_str(actual).unwrap();
        assert_eq!(expected_toml, actual_toml)
    }

    #[test]
    fn json_to_yaml_success() {
        let data = fs::read_to_string("tests/data/test.json").unwrap();
        let yaml_data = json_to_yaml(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.yaml").unwrap();
        compare_yaml_str(&expected, &yaml_data);
    }

    #[test]
    fn json_to_toml_success() {
        let data = fs::read_to_string("tests/data/test.json").unwrap();
        let toml_data = json_to_toml(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.toml").unwrap();
        compare_toml_str(&expected, &toml_data);
    }

    #[test]
    fn yaml_to_json_success() {
        let data = fs::read_to_string("tests/data/test.yaml").unwrap();
        let json_data = yaml_to_json(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.json").unwrap();
        compare_json_str(&expected, &json_data);
    }

    #[test]
    fn yaml_to_toml_success() {
        let data = fs::read_to_string("tests/data/test.yaml").unwrap();
        let toml_data = yaml_to_toml(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.toml").unwrap();
        compare_toml_str(&expected, &toml_data);
    }

    #[test]
    fn toml_to_json_success() {
        let data = fs::read_to_string("tests/data/test.toml").unwrap();
        let json_data = toml_to_json(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.json").unwrap();
        compare_json_str(&expected, &json_data);
    }

    #[test]
    fn toml_to_yaml_success() {
        let data = fs::read_to_string("tests/data/test.toml").unwrap();
        let yaml_data = toml_to_yaml(&data).unwrap();
        let expected = fs::read_to_string("tests/data/test.yaml").unwrap();
        compare_yaml_str(&expected, &yaml_data);
    }
}
