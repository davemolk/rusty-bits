use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use reqwest::Method;

#[derive(Debug, Parser)]
pub struct Args {
    /// URL to request.
    /// If no method is included,
    /// GET will be used.
    #[clap(name="[METHOD] URL", required=true, value_parser=method_and_url_validation)]
    method_and_url: String,

    /// print lots of stuff
    #[clap(short, long)]
    verbose: bool,

    /// dry run -- just print to terminal.
    #[clap(short, long)]
    dry: bool,

    /// retry request as http
    /// if it fails as https
    #[clap(long)]
    http_retry: bool,
}

fn method_and_url_validation(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(anyhow!("must supply a url with optional method preceeding it"))
    }
    Ok(input.into())
}

pub fn run(args: Args) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let (method, url) = parse_method_and_url_arg(&args.method_and_url);
    let https_url = append_https_if_necessary(&url);

    let mut req_builder = match method {
        Method::GET => client.get(&https_url),
        Method::POST => client.post(&https_url),
        Method::PATCH => client.patch(&https_url),
        Method::PUT => client.put(&https_url),
        Method::DELETE => client.delete(&https_url),
        _ => client.get(&https_url),
    };

    req_builder = req_builder
        .headers(default_headers());

    let req = req_builder.build()?;
    if args.verbose || args.dry {
        println!("{:?}", req.version());
        println!("{:?}", req.url().as_str());
        for (h, v) in req.headers() {
            println!("{:?}: {:?}", h, v);
        };
    }
    if args.dry {
        return Ok(());
    }

    let data = client.execute(req)?;

    println!("{:?} {:?} {:?}", data.version(), data.status(), data.status().canonical_reason().unwrap_or_else(|| ""));
    for (h, v) in data.headers() {
        println!("{:?}: {:?}", h, v)
    }
    println!("Content-Length: {:?}", data.content_length().unwrap_or_default());
    println!();
    println!("{}", data.text()?);
    Ok(())
}

fn parse_method_and_url_arg(method_and_url: &str) -> (Method, String) {
    // we know method_and_url can't have length 0 because of our input
    // validation w/ clap. therefore any split we do will have a length
    // of at least 1.
    let split_arg: Vec<_> = method_and_url.split(' ').collect();
    let url: String;
    let method: Method;
    // just url
    if split_arg.len() == 1 {
        url = split_arg[0].to_string();
        method = Method::GET;
    } else {
        url = split_arg[1].to_string();
        method = match split_arg[0].to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PATCH" => Method::PATCH,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            _ => Method::GET,
        };
    }
    (method, url)
}

// we can try for http if this doesn't work
fn append_https_if_necessary(url_arg: &str) -> String {
    if url_arg.starts_with("https") {
        return url_arg.into();
    }
    format!("https://{}", url_arg)
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("github.com/davemolk/rusty-bits/req"));
    headers
}