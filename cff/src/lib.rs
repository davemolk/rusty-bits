use clap::Parser;
use anyhow::{anyhow, Result};
use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::collections::HashMap;


#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
enum Format {
    JSON,
    Yaml,
    Toml,
    CSV,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// path to source file
    #[clap(short, long)]
    source_path: PathBuf,

    /// path to destination file.
    /// if no path is provided,
    /// will output to stdout.
    #[clap(short, long)]
    dest_path: Option<PathBuf>,

    /// conversion to do:
    ///     JY  json to yaml
    ///     YJ  yaml to json
    ///     JT  json to toml
    ///     TJ  toml to json
    ///     YT  yaml to toml
    ///     TY  toml to yaml
    #[clap(required=true, value_parser = parse_conversion)]
    conversion: (Format, Format),
}

fn parse_conversion(conversion: &str) -> Result<(Format, Format)> {
    let c = match conversion.to_uppercase().as_str() {
        "JY" => (Format::JSON, Format::Yaml),
        "JT" => (Format::JSON, Format::Toml),
        "YJ" => (Format::Yaml, Format::JSON),
        "YT" => (Format::Yaml, Format::Toml),
        "TJ" => (Format::Toml, Format::JSON),
        "TY" => (Format::Toml, Format::JSON),
        _ => {
            return Err(anyhow!("conversion unsupported"));
        }
    };
    Ok(c)
}

fn yaml_to_json(src_path: &Path) -> Result<String> {
    let yaml_file = File::open(src_path)?;
    let json_data: serde_json::Value = serde_yaml::from_reader(BufReader::new(yaml_file))?;
    Ok(serde_json::to_string_pretty(&json_data)?)
}

fn yaml_to_toml(src_path: &Path) -> Result<String> {
    let yaml_file = File::open(src_path)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_reader(BufReader::new(yaml_file))?;
    Ok(toml::to_string(&yaml_value)?)
}

fn json_to_yaml(src_path: &Path) -> Result<String> {
    let json_file = File::open(src_path)?;
    let yaml_data: serde_yaml::Value = serde_json::from_reader(BufReader::new(json_file))?;
    Ok(serde_yaml::to_string(&yaml_data)?)
}

fn json_to_toml(src_path: &Path) -> Result<String> {
    let json_file = File::open(src_path)?;
    let json_value: serde_json::Value = serde_json::from_reader(BufReader::new(json_file))?;
    Ok(toml::to_string(&json_value)?)
}

fn toml_to_json(src_path: &Path) -> Result<String> {
    let toml_data = fs::read_to_string(src_path)?;
    let json_data: serde_json::Value = toml::from_str(&toml_data)?;
    Ok(serde_json::to_string_pretty(&json_data)?)
}

fn toml_to_yaml(src_path: &Path) -> Result<String> {
    let toml_data = fs::read_to_string(src_path)?;
    let yaml_data: serde_yaml::Value = toml::from_str(&toml_data)?;
    Ok(serde_yaml::to_string(&yaml_data)?)
}

pub fn run(args: Args) -> Result<()> {
    let mut conversions: HashMap<(Format, Format), fn(&Path) -> Result<String>> = HashMap::new();
    conversions.insert((Format::Yaml, Format::JSON), yaml_to_json); 
    conversions.insert((Format::Yaml, Format::Toml), yaml_to_toml);
    conversions.insert((Format::JSON, Format::Yaml), json_to_yaml);
    conversions.insert((Format::JSON, Format::Toml), json_to_toml);
    conversions.insert((Format::Toml, Format::JSON), toml_to_json);
    conversions.insert((Format::Toml, Format::Yaml), toml_to_yaml);

    let conversion_fn = conversions.get(&args.conversion).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Conversion function not found")
    })?;
    let data = conversion_fn(&args.source_path)?;
    if let Some(path) = args.dest_path {
        write_data(&path, &data)?;
    } else {
        println!("{}", &data);
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
        let path = PathBuf::from("tests/data/test.json");
        let yaml_data = json_to_yaml(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.yaml").unwrap();
        compare_yaml_str(&expected, &yaml_data);
    }

    #[test]
    fn json_to_toml_success() {
        let path = PathBuf::from("tests/data/test.json");
        let toml_data = json_to_toml(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.toml").unwrap();
        compare_toml_str(&expected, &toml_data);
    }

    #[test]
    fn yaml_to_json_success() {
        let path = PathBuf::from("tests/data/test.yaml");
        let json_data = yaml_to_json(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.json").unwrap();
        compare_json_str(&expected, &json_data);
    }

    #[test]
    fn yaml_to_toml_success() {
        let path = PathBuf::from("tests/data/test.yaml");
        let toml_data = yaml_to_toml(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.toml").unwrap();
        compare_toml_str(&expected, &toml_data);
    }

    #[test]
    fn toml_to_json_success() {
        let path = PathBuf::from("tests/data/test.toml");
        let json_data = toml_to_json(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.json").unwrap();
        compare_json_str(&expected, &json_data);
    }

    #[test]
    fn toml_to_yaml_success() {
        let path = PathBuf::from("tests/data/test.toml");
        let yaml_data = toml_to_yaml(&path).unwrap();
        let expected = fs::read_to_string("tests/data/test.yaml").unwrap();
        compare_yaml_str(&expected, &yaml_data);
    }
}
