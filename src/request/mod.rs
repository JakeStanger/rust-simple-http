use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub use get::Get;
pub use patch::Patch;
pub use post::Post;
pub use delete::Delete;

use crate::buffer_utils::{find_substring, split};
use crate::{Opts, Response};

mod get;
mod patch;
mod post;
mod delete;

pub type ResponseResult<'a> = Result<Response<'a>, Box<dyn Error>>;
pub trait RequestHandler {
    /// Gets a response for a request.
    ///
    /// `Err` responses should only be given for uncaught server-side problems,
    /// meaning HTTP error codes ie 400, 404, 501 etc are all `Ok` responses
    fn get_response<'a>(req: &'a Request, opts: Arc<Opts>) -> ResponseResult<'a>;
}

pub struct ReqStatusLine<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub http_version: &'a str,
}

pub struct Request<'a> {
    pub status_line: ReqStatusLine<'a>,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<&'a [u8]>,
}

#[derive(Debug)]
pub struct ComplexHeader<'a> {
    pub value: &'a str,
    pub extras: HashMap<&'a str, &'a str>,
}

impl Request<'_> {
    /// Converts a raw request string into a `Request` struct
    pub fn from_bytes(buffer: &[u8]) -> Request {
        let buffer_size = buffer.len();

        let end_status_line = find_substring(&buffer, b"\r\n").unwrap();
        let status_line = &buffer[0..end_status_line];

        let end_headers = find_substring(&buffer, b"\r\n\r\n").unwrap_or(buffer_size);
        let headers = Request::parse_headers(&buffer[end_status_line + 2..end_headers]);

        let content_length = headers.get("Content-Length").map_or(buffer_size, |&size| {
            size.parse::<usize>().unwrap() + end_headers
        });

        const NEWLINE_OFFSET: usize = 4;

        let body = if end_headers < buffer_size {
            Some(
                &buffer[end_headers + NEWLINE_OFFSET
                    ..min(content_length + NEWLINE_OFFSET, buffer_size)],
            )
        } else {
            None
        };

        Request {
            status_line: Request::parse_status_line(&status_line),
            headers,
            body,
        }
    }

    fn parse_status_line(status_line: &[u8]) -> ReqStatusLine {
        let status_line = std::str::from_utf8(status_line).unwrap();

        let mut parts = status_line.split_whitespace();
        let method = parts.next().unwrap();
        let uri = parts.next().unwrap();
        let http_version = parts.next().unwrap();

        ReqStatusLine {
            method,
            uri,
            http_version,
        }
    }

    pub fn parse_headers(headers: &[u8]) -> HashMap<&str, &str> {
        let mut map = HashMap::new();
        for header in split(headers, b"\r\n") {
            let header = std::str::from_utf8(header).unwrap();

            let mut split = header.split(": ");
            let key = split.next().unwrap();
            let value = split.next().unwrap();
            map.insert(key, value);
        }

        map
    }

    /// Splits a multi-part header that consists of
    /// semicolon-separated `key=value` pairs.
    pub fn parse_complex_header(header: &str) -> ComplexHeader {
        let sections = header
            .split(";")
            .map(|section| section.trim())
            .collect::<Vec<_>>();

        let value = sections[0];

        let extras = sections[1..]
            .into_iter()
            .map(|&part| {
                let (key, value) = part.split_once("=").unwrap_or((part, ""));

                let value_length = value.len();
                let value: &str = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value_length - 1]
                } else {
                    value
                };

                (key, value)
            })
            .collect::<HashMap<&str, &str>>();

        ComplexHeader { value, extras }
    }
}
