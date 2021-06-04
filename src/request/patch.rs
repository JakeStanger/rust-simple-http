use std::fs;
use std::sync::Arc;

use urlencoding::decode;

use crate::path_utils::{get_path, is_filepath};
use crate::request::{Request, RequestHandler, ResponseResult};
use crate::{multipart, Opts, Response};
use std::path::PathBuf;

pub struct Patch;

impl RequestHandler for Patch {
    fn get_response<'a>(req: &'a Request<'a>, opts: Arc<Opts>) -> ResponseResult {
        if req.body.is_none() {
            return Ok(Response::error(400, Some("Missing request body")));
        }

        let save_path = get_path(
            opts.directory.as_str(),
            decode(req.status_line.uri).unwrap().as_str(),
        );

        let is_filepath = is_filepath(&save_path);

        let content_type =
            Request::parse_complex_header(req.headers.get("Content-Type").unwrap_or(&""));

        let mut file_paths = Vec::new();

        if content_type.value == "multipart/form-data" {
            let files = multipart::parse(req, &content_type)?;

            if is_filepath && files.len() > 1 {
                return Ok(Response::error(
                    400,
                    Some(
                        format!(
                            "Path <code>{}</code> cannot be a filename",
                            save_path.to_str().unwrap()
                        )
                        .as_str(),
                    ),
                ));
            }

            if !save_path.exists() {
                let requested_path = req.status_line.uri.parse::<PathBuf>()?;
                let rel_path = requested_path.parent().unwrap();

                return Ok(Response::error(
                    404,
                    Some(
                        format!(
                            "Directory <code>{}</code> does not exist",
                            rel_path.to_str().unwrap()
                        )
                        .as_str(),
                    ),
                ));
            }

            for file in files {
                let rel_path = get_path(
                    req.status_line
                        .uri
                        .strip_prefix("/")
                        .unwrap_or(req.status_line.uri),
                    file.name.as_str(),
                );
                let file_path = save_path.join(file.name);

                if file_path.exists() {
                    fs::write(file_path, file.body)?;
                } else {
                    return Ok(Response::error(
                        404,
                        Some(
                            format!(
                                "File <code>{}</code> does not exist",
                                rel_path.to_str().unwrap()
                            )
                            .as_str(),
                        ),
                    ));
                }

                file_paths.push(rel_path);
            }
        } else {
            let req_path = req
                .status_line
                .uri
                .strip_prefix("/")
                .unwrap_or(req.status_line.uri);

            if !is_filepath {
                return Ok(Response::error(
                    400,
                    Some(
                        format!(
                            "Path <code>{}</code> is not a filename",
                            save_path.to_str().unwrap()
                        )
                        .as_str(),
                    ),
                ));
            }

            let rel_path = req_path.parse().unwrap();

            fs::write(save_path, req.body.unwrap())?;
            file_paths.push(rel_path);
        }

        let host = get_host(req, opts);

        let links: Vec<String> = file_paths
            .into_iter()
            .map(|name| format!("http://{}/{}", host, name.to_str().unwrap()))
            .collect();

        Ok(Response::ok(
            201,
            req.status_line.uri.parse()?,
            (links.join("\n") + "\n").as_bytes().to_vec(),
        ))
    }
}

fn get_host(req: &Request, opts: Arc<Opts>) -> String {
    req.headers.get("Host").map_or_else(
        || format!("{}:{}", opts.host, opts.port),
        |&host| host.to_string(),
    )
}
