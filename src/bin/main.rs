use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use clap::Clap;

use http_impl_demo::request::{Get, Post, Patch, Delete, Request, RequestHandler};
use http_impl_demo::{Opts, Response, ThreadPool};
use std::io;

fn main() {
    let matches: Arc<Opts> = Arc::new(Opts::parse());

    let socket = format!("{}:{}", matches.host, matches.port);

    let listener = TcpListener::bind(&socket).expect("Could not bind to socket");
    let pool = ThreadPool::new(16, matches.clone());

    println!("Serving on http://{}", socket);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|opts| {
            handle_connection(stream, opts).unwrap();
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, opts: Arc<Opts>) -> Result<(), io::Error> {
    const BUFFER_SIZE: usize = 1_048_576; // 1MB
    let mut buffer = [0; BUFFER_SIZE];

    stream.read(&mut buffer)?;

    let req = Request::from_bytes(&buffer);

    let response = match match req.status_line.method {
        "GET" => Get::get_response(&req, opts),
        "POST" => Post::get_response(&req, opts),
        "PATCH" => Patch::get_response(&req, opts),
        "DELETE" => Delete::get_response(&req, opts),
        _ => Ok(Response::error(
            501,
            Some(format!("Method {} is not supported", req.status_line.method).as_str()),
        )),
    } {
        Ok(res) => res,
        Err(err) => {
            println!("ERR: {}", err);
            Response::error(500, Some(err.to_string().as_str()))
        }
    };

    stream.write(response.to_bytes().as_slice())?;
    stream.flush()?;

    log(&req, &response);

    Ok(())
}

fn log(req: &Request, res: &Response) {
    let host = req.headers.get("Host").unwrap_or(&"?????");
    let date = res.headers.get("Date").unwrap();
    let method = req.status_line.method;
    let code = res.status_code;
    let uri = req.status_line.uri;

    println!("[{}]\t{}\t{} {} {}", date, host, method, code, uri);
}
