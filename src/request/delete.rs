use std::fs;
use std::sync::Arc;

use urlencoding::{decode};

use crate::path_utils::{get_path, is_filepath};
use crate::request::{Request, RequestHandler, ResponseResult};
use crate::{Opts, Response};

pub struct Delete;

impl RequestHandler for Delete {
    fn get_response<'a>(req: &'a Request<'a>, opts: Arc<Opts>) -> ResponseResult {
        let delete_path = get_path(
            opts.directory.as_str(),
            decode(req.status_line.uri).unwrap().as_str(),
        );

        let is_filepath = is_filepath(&delete_path);

        if !is_filepath || delete_path.is_dir() {
            return Ok(Response::error(400, Some("You can only delete individual files")));
        }

        if delete_path.exists() {
            fs::remove_file(delete_path)?;

            Ok(Response::ok(
            200,
            req.status_line.uri.parse()?,
            "OK".as_bytes().to_vec(),
        ))
        } else {
            return Ok(Response::error(400, Some("File does not exist")));
        }
    }
}
