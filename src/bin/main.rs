use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use http_impl_demo::{Request, Response, ThreadPool, Opts};
use std::sync::{Arc};

use clap::{Clap};

fn main() {
    let matches: Arc<Opts> = Arc::new(Opts::parse());

    let socket = format!("{}:{}", matches.host, matches.port);

    let listener = TcpListener::bind(&socket).unwrap();
    let pool = ThreadPool::new(16, &matches);

    println!("Serving on http://{}", socket);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|opts| {
            handle_connection(stream, opts);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, opts: &Opts) {
    const BUFFER_SIZE: usize = 8096;
    let mut buffer = [0; BUFFER_SIZE];

    stream.read(&mut buffer).unwrap();

    let req = String::from_utf8_lossy(
        &buffer[0..buffer.iter().position(|&b| b == 0).unwrap_or(BUFFER_SIZE)],
    );

    let req = Request::from_string(&req);

    let response = match req.status_line.method {
        "GET" => get(&req, opts),
        _ => Response::error(501),
    };

    stream.write(response.to_bytes().as_slice()).unwrap();
    stream.flush().unwrap();

    log(&req, &response);
}

/// Takes a GET `request` and returns a `response`.
fn get<'a>(request: &Request, opts: &Opts) -> Response<'a> {
    let resource = if request.status_line.uri != "/" {
        request.status_line.uri
    } else {
        "/index.html"
    };

    let resource_path = format!("{}{}", opts.directory, resource);
    let path = Path::new(resource_path.as_str());

    let path: PathBuf = if path.is_dir() {
        path.join("index.html")
    } else {
        path.to_path_buf()
    };

    if path.exists() {
        let content = fs::read(&path).unwrap();
        Response::ok(path, content)
    } else {
        Response::error(404)
    }
}

fn log(req: &Request, res: &Response) {
    let host = req.headers.get("Host").unwrap_or(&"?????");
    let date = res.headers.get("Date").unwrap();
    let method = req.status_line.method;
    let code = res.status_code;
    let uri = req.status_line.uri;

    println!("[{}]\t{}\t{} {} {}", date, host, method, code, uri);
}
