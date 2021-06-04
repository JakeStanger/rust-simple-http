use std::fs;
use std::sync::Arc;

use urlencoding::decode;

use crate::path_utils::get_path;
use crate::request::{Request, RequestHandler, ResponseResult};
use crate::{Opts, Response};

pub struct Get;

impl RequestHandler for Get {
    fn get_response<'a>(req: &'a Request, opts: Arc<Opts>) -> ResponseResult<'a> {
        let path = get_path(
            opts.directory.as_str(),
            decode(req.status_line.uri).unwrap().as_str(),
        );

        // default to index.html for directories
        let file_path = if path.is_dir() {
            path.join("index.html")
        } else {
            path.to_path_buf()
        };

        if file_path.exists() {
            let content = fs::read(&file_path)?;
            Ok(Response::ok(200, file_path, content))
        } else if path.is_dir() {
            let dir_contents = path
                .read_dir()?
                .map(|file| file.unwrap().file_name())
                .collect::<Vec<_>>();

            let contents_html = dir_contents
                .iter()
                .map(|file| {
                    format!(
                        "<li><a href=\"{}\"</a>{}</li>",
                        get_path(req.status_line.uri, file.to_str().unwrap())
                            .to_str()
                            .unwrap(),
                        file.to_str().unwrap()
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            Ok(Response::ok(
                200,
                path,
                format!(
                    "<h1>Directory Listing</h1><ul><li><a href=\"{}\">..</a></li>{}</ul>",
                    get_path(req.status_line.uri, "..").to_str().unwrap(),
                    contents_html
                )
                .into_bytes(),
            ))
        } else {
            Ok(Response::error(
                404,
                Some(format!("File <code>{}</code> does not exist", req.status_line.uri).as_str()),
            ))
        }
    }
}
