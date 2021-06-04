use crate::buffer_utils::{find_substring, split};
use crate::{UploadFile};
use crate::request::{ComplexHeader, Request};
use crate::path_utils::get_filename_or_fallback;

/// Parses a `multipart/*` request and returns a list of files to be handled.
pub fn parse<'a>(req: &'a Request, content_type: &'a ComplexHeader) -> Result<Vec<UploadFile<'a>>, &'a str> {
    if !content_type.extras.contains_key("boundary") {
        return Err("Malformed multipart request; missing boundary")
    }

    let base_boundary = content_type.extras.get("boundary").unwrap();
    let boundary = "--".to_owned() + base_boundary + "\r\n";
    let end_boundary = "--".to_owned() + base_boundary + "--\r\n";

    let body = req.body.unwrap();
    let body = body.strip_prefix(boundary.as_bytes()).unwrap_or(body);
    let body = body.strip_suffix(end_boundary.as_bytes()).unwrap_or(body);

    let parts = split(body, boundary.as_bytes())
        .iter()
        .map(|&part| {
            const PREDICATE: &[u8] = "\r\n\r\n".as_bytes();
            let (headers, body) = part.split_at(find_substring(part, PREDICATE).unwrap());
            let body = &body[PREDICATE.len()..];
            (headers, body)
        })
        .collect::<Vec<(&[u8], &[u8])>>();

    let files = parts
        .into_iter()
        .map(|(headers, body)| {
            let headers = Request::parse_headers(headers);
            let disposition =
                Request::parse_complex_header(headers.get("Content-Disposition").unwrap_or(&""));

            let filename = get_filename_or_fallback(disposition.extras.get("filename"));

            UploadFile {
                name: filename,
                body,
            }
        })
        .collect();

    Ok(files)
}