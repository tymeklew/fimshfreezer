use log::{debug, error, info};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
const R: u8 = b'\r';
const N: u8 = b'\n';

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
}
#[derive(Debug)]
struct Request {}
impl Request {
    fn parse<T: Read + std::fmt::Debug>(input: T) -> Result<Self> {
        let reader = BufReader::new(input);
        let mut temp = Vec::new();
        let mut bytes = reader.bytes();
        let mut top = Vec::new();
        let body: Vec<u8>;
        for b in bytes.by_ref() {
            if let Ok(b) = b {
                if temp.len() == 4 {
                    let headers = Self::parse_headers(top);
                    let size = headers
                        .get("Content-Length")
                        .unwrap_or(&"0".to_string())
                        .parse::<usize>()
                        .unwrap_or(0);
                    body = bytes.take(size - 1).map(|f| f.unwrap()).collect();
                    info!("Headers : {:?}", headers);
                    info!("Body : {}", String::from_utf8_lossy(&body));
                    break;
                } else if b == R || b == N {
                    temp.push(b);
                } else {
                    temp = Vec::new();
                }
                top.push(b);
                println!("{:?}", b);
            }
        }

        todo!()
    }
    fn parse_headers(input: Vec<u8>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        for line in input.lines() {
            println!("LIne : {:?}", line);
            match line {
                Ok(line) => match line.split_once(":") {
                    Some((k, v)) => {
                        headers.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    None => continue,
                },
                Err(_) => continue,
            }
        }
        return headers;
    }
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    info!("Starting server on port 8080");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream)?,
            Err(e) => error!("{e}"),
        }
    }
    Ok(())
}
fn handle_stream(stream: TcpStream) -> Result<()> {
    Request::parse(stream).unwrap();
    Ok(())
}
