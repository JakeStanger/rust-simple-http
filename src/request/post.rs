use std::fs;
use std::sync::Arc;

use mime_guess::get_mime_extensions_str;
use urlencoding::{decode};

use crate::path_utils::{get_path, is_filepath, random_string, sanitise};
use crate::request::{Request, RequestHandler, ResponseResult};
use crate::{multipart, Opts, Response};

pub struct Post;

impl RequestHandler for Post {
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
                fs::create_dir_all(&save_path)?;
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

                if !file_path.exists() {
                    fs::write(file_path, file.body)?;
                } else {
                    return Ok(Response::error(
                        400,
                        Some(
                            format!(
                                "File <code>{}</code> already exists",
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

            let (filename, rel_path) = if is_filepath {
                fs::create_dir_all(&save_path.parent().unwrap_or(&save_path))?;

                let filename = save_path.file_name().unwrap().to_string_lossy().to_string();

                (filename, req_path.parse().unwrap())
            } else {
                fs::create_dir_all(&save_path)?;

                let filename = req.headers.get("X-File-Name").map_or_else(
                    || {
                        let filename = random_string(6);
                        let extension =
                            get_mime_extensions_str(req.headers.get("Content-Type").unwrap_or(&""))
                                .unwrap_or(&["txt"])[0];
                        format!("{}.{}", filename, extension)
                    },
                    |&name| sanitise(name),
                );

                let path = get_path(req_path, &filename.as_str());

                (filename, path)
            };

            let file_path = if !is_filepath {
                save_path.join(filename)
            } else {
                save_path
            };

            fs::write(file_path, req.body.unwrap())?;
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
