use std::collections::HashMap;

pub struct RequestLine<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub http_version: &'a str,
}

pub struct Request<'a> {
    pub status_line: RequestLine<'a>,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: Option<Vec<u8>>,
}

impl Request<'_> {
    /// Converts a raw request string into a `Request` struct
    pub fn from_string(req: &str) -> Request {
        let (req_line, rest) = req.split_once("\r\n").unwrap();
        let req_line = Request::parse_request_line(req_line);

        let (headers_str, body) = rest.split_once("\r\n\r\n").unwrap();
        let headers = Request::parse_headers(headers_str);

        Request {
            status_line: req_line,
            headers,
            body: if body.len() > 0 {
                Some(Vec::from(body.as_bytes()))
            } else {
                None
            },
        }
    }

    fn parse_request_line(req_line: &str) -> RequestLine {
        let mut parts = req_line.split_whitespace();
        let method = parts.next().unwrap();
        let uri = parts.next().unwrap();
        let http_version = parts.next().unwrap();

        RequestLine {
            method,
            uri,
            http_version,
        }
    }

    fn parse_headers(headers: &str) -> HashMap<&str, &str> {
        let mut map = HashMap::new();
        for header in headers.split('\n') {
            let mut split = header.strip_suffix('\r').unwrap_or(header).split(": ");
            let key = split.next().unwrap();
            let value = split.next().unwrap();
            map.insert(key, value);
        }

        map
    }
}
