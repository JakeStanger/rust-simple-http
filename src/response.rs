use std::collections::HashMap;

use chrono::Local;

use crate::{HTTP_VERSION};
use std::path::PathBuf;

pub struct Response<'a> {
    pub http_version: &'a str,
    pub status_code: u16,
    pub reason_phrase: &'a str,
    pub headers: HashMap<&'a str, String>,
    pub body: Option<Vec<u8>>,
}

impl Response<'_> {
    /// Converts the response object to a byte vector.
    ///
    /// The vec contains the formatted HTTP response
    /// which can be sent back to the client.
    pub fn to_bytes(&self) -> Vec<u8> {
        let status_line = format!(
            "{} {} {}",
            self.http_version, self.status_code, self.reason_phrase
        );
        let headers = self
            .headers
            .iter()
            .map(|(&key, value)| format!("{}: {}", key, value))
            .collect::<Vec<String>>()
            .join("\r\n");

        let mut response = format!("{}\r\n{}\r\n\r\n", status_line, headers)
            .as_bytes()
            .to_vec();
        let empty_body = &Vec::new();
        let body = self.body.as_ref().unwrap_or(empty_body);

        response.extend_from_slice(body);
        response
    }

    /// Gets a map of base response headers
    pub fn get_headers<'a>(content_length: usize, path: &str) -> HashMap<&'a str, String> {
        let mut headers = HashMap::new();

        headers.insert("Server", "SimpleHTTP/0.1 Rust".to_string());
        headers.insert("Connection", "Keep-Alive".to_string());
        headers.insert("Date", format!("{}", Local::now().to_rfc2822()));
        headers.insert("Content-Length", format!("{}", content_length));
        headers.insert(
            "Content-Type",
            mime_guess::from_path(path)
                .first_or("text/html".parse().unwrap())
                .to_string(),
        );

        headers
    }

    /// Returns an HTTP OK response
    pub fn ok<'a>(code: u16, path: PathBuf, content: Vec<u8>) -> Response<'a> {
        // ok responses should always have a 200-code
        assert!(code >= 200 && code <= 300);

        const STATUS_CODE: u16 = 200;

        Response {
            http_version: HTTP_VERSION,
            status_code: STATUS_CODE,
            reason_phrase: Response::reason_phrase(STATUS_CODE),

            // error path here doesn't matter as we just want to get html mimetype
            headers: Response::get_headers(content.len(), path.to_str().unwrap()),
            body: Some(content),
        }
    }

    /// Returns an error response for the given error code.
    pub fn error<'a>(status_code: u16, details: Option<&str>) -> Response<'a> {
        let content = Response::get_error_html(status_code, details);

        Response {
            http_version: HTTP_VERSION,
            status_code,
            reason_phrase: Response::reason_phrase(status_code),

            // error path here doesn't matter as we just want to get html mimetype
            headers: Response::get_headers(content.len(), "error.html"),
            body: Some(content),
        }
    }

    /// Loads the HTML page for the given error code
    fn get_error_html(code: u16, details: Option<&str>) -> Vec<u8> {
        format!(
            "<!DOCTYPE html> \
            <html lang=\"en\"> \
            <head> \
                <meta charset=\"UTF-8\"> \
                <title>{code} | {reason_phrase}</title> \
            </head> \
            <body> \
                <h1>{reason_phrase}</h1> \
                <p>{details}</p> \
            </body> \
            </html>",
            code = code,
            reason_phrase = Response::reason_phrase(code),
            details = details.unwrap_or("")
        )
        .as_bytes()
        .to_vec()
    }

    /// Gets the reason phrase string for an error code
    pub fn reason_phrase<'a>(code: u16) -> &'a str {
        match code {
            200 => "OK",
            400 => "BAD REQUEST",
            404 => "NOT FOUND",
            500 => "INTERNAL SERVER ERROR",
            501 => "NOT IMPLEMENTED",
            _ => "UNKNOWN ERROR",
        }
    }
}
