use core::str;
use std::fs;
use std::path::{PathBuf, Path};
use std::str::FromStr;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE};
use reqwest::{
    Method,
    blocking::{Request, Client, multipart},
};
use url::Url;
use serde_json::Value;

const USER_AGENT_DEFAULT: &str = "github.com/davemolk/rusty-bits/rq";

#[derive(Debug, Parser, Default)]
pub struct Args {
    /// URL to request
    #[clap(required=true)]
    url: String,

    /// defaults to GET if a value is not supplied
    #[clap(short, long, default_value = "GET", value_parser = parse_method)]
    method: Method,

    /// basic auth
    /// (formatted as user:pw)
    #[clap(long)]
    basic: Option<String>,

    /// bearer auth
    #[clap(long)]
    bearer: Option<String>,

    /// data for request body.
    /// preface a file_path with @.
    /// 
    /// -d "my string"
    /// -d '{"my": "json"}'
    /// -d @my_file
    #[clap(short, long)]
    data: Option<String>,

    /// send a multipart/form-data body
    #[clap(long)]
    form: Option<String>,

    /// print request and response info to os.Stdout
    #[clap(short, long)]
    verbose: bool,

    /// print the request info to os.Stdout and exit
    /// without making the request
    #[clap(long)]
    debug: bool,

    /// set timeout for request
    /// in seconds.
    #[clap(short, long="timeout")]
    timeout_seconds: Option<u64>,

    /// header(s) to include.
    /// preface a file_path with @
    /// to set headers from a json_file.
    /// e.g. { "Accept-Language": "en-US" }
    /// 
    /// -H Accept-Language=en-US
    #[clap(short='H', long)]
    headers: Option<Vec<String>>,

    /// cookie(s) to include.
    /// preface a file_path with @
    /// to set cookies from a json_file.
    /// e.g. { "foo": "bar" }
    /// 
    /// send multiple as so:
    /// -c "foo=bar; chocolate=chip"
    #[clap(short, long)]
    cookies: Option<String>,

    /// proxy to use
    #[clap(short, long)]
    proxy: Option<String>,

    /// don't follow redirects
    #[clap(long="no-redirects")]
    redirects: bool,

    /// only use HTTP/2
    #[clap(long)]
    http2: bool,

    /// set a custom user-agent.
    /// will default to repo path if
    /// none is provided. 
    /// 
    /// note: setting this field via headers
    /// will override any user-agent set with
    /// this option.
    #[clap(long)]
    user_agent: Option<String>,

    /// download file to provided path.
    #[clap(long)]
    download: Option<String>,

    /// pretty-print json file.
    #[clap(long="pp")]
    pretty_print: bool,
}

pub fn run(mut args: Args) -> Result<()> {
    let client = build_client(&mut args)?;
    let req = build_request(&mut args, &client)?;    

    if args.verbose || args.debug {
        println!("{:?}", req.version());
        println!("{:?}", req.url().as_str());
        println!("{:?}", req.method());
        for (h, v) in req.headers() {
            println!("{:?}: {:?}", h, v);
        };
        if let Some(t) = req.timeout() {
            println!("timeout: {:?}", t)
        }
        println!();
    }

    if args.debug {
        return Ok(());
    }

    let mut data = client.execute(req)?;

    if !data.status().is_success() {
        eprintln!("status: {:?}",data.status().canonical_reason())
    }

    if args.verbose {
        println!("{:?} {:?} {:?}", data.version(), data.status(), data.status().canonical_reason().unwrap_or_default());
        for (h, v) in data.headers() {
            println!("{:?}: {:?}", h, v)
        }
        println!();
    }

    if let Some(download_path) = args.download {
        let mut file = fs::File::create(&download_path)?;
        println!("downloading file...");
        file.write_all(&mut data.bytes()?)?;
        return Ok(());
    }

    if args.pretty_print {
        let json_res: Value = serde_json::from_reader(&mut data)?;
        match serde_json::to_string_pretty(&json_res) {
            Ok(pp) => println!("{pp}"),
            // just print it
            Err(_) => println!("{:?}", data.text()),
        }
    } else {
        println!("{}", data.text()?);
    }
    Ok(())
}

fn build_client(args: &mut Args) -> Result<Client> {
    let mut client = reqwest::blocking::ClientBuilder::new();

    client = if let Some(ua) = &args.user_agent {
        client.user_agent(ua)
    } else {
        client.user_agent(USER_AGENT_DEFAULT)
    };
    
    if !args.redirects {
        client = client.redirect(reqwest::redirect::Policy::none());
    }

    if args.http2 {
        client = client.http2_prior_knowledge();
    }

    if let Some(proxy) = &args.proxy {
        client = client.proxy(reqwest::Proxy::all(proxy).with_context(|| format!("invalid proxy: {}", proxy))?);
    }

    let client = client.build().with_context(|| "building client")?;
    Ok(client)
}

fn build_request(args: &mut Args, client: &Client) -> Result<Request> {    
    let url = Url::parse(&args.url)
        .with_context(|| format!("{} cannot be parsed as url", args.url))?;

    let mut req_builder = match args.method {
        Method::GET => client.get(url),
        Method::HEAD => client.head(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::PATCH => client.patch(url),
        Method::DELETE => client.delete(url),
        _ => client.get(url),
    };

    req_builder = req_builder
        .headers(add_headers(&args.headers).with_context(|| "adding headers")?);
        
    if let Some(cookies) = &args.cookies {
        let cookie_header = add_cookies(&cookies)?;
        req_builder = req_builder.headers(cookie_header);
    }   

    if let Some(basic) = &args.basic {
        let user_pw: Vec<&str> = basic.split(':').collect();
        if user_pw.len() < 2 {
            return Err(anyhow!("malformed basic auth: {}", basic));
        }
        req_builder = req_builder.basic_auth(user_pw[0], Some(user_pw[1]));
    }

    if let Some(bearer) = &args.bearer {
        req_builder = req_builder.bearer_auth(bearer);
    }

    if let Some(t) = args.timeout_seconds {
        req_builder = req_builder.timeout(std::time::Duration::from_secs(t));
    }

    if let Some(d) = &args.data {
        if d.starts_with('@') {
            let data_path = d.clone().split_off(1);
            let file = std::fs::File::open(&data_path)
                .with_context(|| format!("failed to open {data_path}"))?;
            req_builder = req_builder.body(file);
        } else {
            req_builder = req_builder.body(d.clone());
        }        
    }

    // todo: allow files for form data too
    if let Some(form_data) = &args.form {
        let mut form = multipart::Form::new();
        let form_json: Value = serde_json::from_str(form_data).
            with_context(|| format!("bad form data: {}", form_data))?;
        if let Some(object) = form_json.as_object() {
            for (key, value) in object {
                if let Some(form_value) = value.as_str() {
                    // check if it's a file first, otherwise use as text
                    let path = Path::new(form_value);
                    if path.exists() {
                        form = form.file(key.to_owned(), path)?;
                    } else {
                        form = form.text(key.to_owned(), form_value.to_owned());
                    }
                }
            }
        }
        req_builder = req_builder.multipart(form);
    }

    let req = req_builder.build().with_context(|| "building request")?;
    Ok(req)
}

fn parse_method(method: &str) -> Result<Method> {
    let method = match method.to_uppercase().as_str() {
        "GET" => Method::GET,
        "HEAD" => Method::HEAD,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "PATCH" => Method::PATCH,
        "DELETE" => Method::DELETE,
        _ => Method::GET,
    };
    Ok(method)
}

fn add_headers(headers_to_add: &Option<Vec<String>>) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    if let Some(headers) = headers_to_add {
        for header in headers {
            if header.starts_with('@') {
                // load from file
                let path = PathBuf::from(header.clone().split_off(1));
                add_headers_from_file(&mut header_map, path)?;
            } else {
                let header_pair: Vec<_> = header.split('=').collect();
                if header_pair.len() != 2 {
                    return Err(anyhow!("malformed header: {:?}", header_pair));
                }
                let header_key = HeaderName::from_str(header_pair[0])?;
                let header_value = HeaderValue::from_str(header_pair[1])?;
                header_map.insert(header_key, header_value);
            }
        }
    }
    Ok(header_map)
}

fn add_headers_from_file(headers: &mut HeaderMap, path: PathBuf) -> Result<()> {
    let file = std::fs::File::open(&path)?;
    let header_json: Value = serde_json::from_reader(file)
        .with_context(|| format!("bad json format from header file: {:?}", path))?;
    if let Some(json_obj) = header_json.as_object() {
        for (key, value) in json_obj {
            if let Some(v) = value.as_str() {
                let header_key = HeaderName::from_str(key)?;
                headers.insert(header_key, HeaderValue::from_str(v)?);
            } else {
                return Err(anyhow!("{value} is not a proper header value"))
            }    
        }
    }
    Ok(())
}

fn add_cookies(cookies: &str) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    if cookies.starts_with('@') {
        let (_, cookie_path) = cookies.split_at(1);
        let path = PathBuf::from(cookie_path);
        let file_cookies = fs::read_to_string(path)?;
        header_map.insert(COOKIE, HeaderValue::from_str(&file_cookies)?);
    } else {
        header_map.insert(COOKIE, HeaderValue::from_str(&cookies)?);
    }
    Ok(header_map)
}
