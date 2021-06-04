use clap::{AppSettings, Clap};

pub use response::Response;
pub use thread_pool::ThreadPool;

mod thread_pool;
pub mod request;
mod response;
pub mod buffer_utils;
pub mod multipart;
pub mod path_utils;

pub const ERROR_PATH: &str = "errors";
pub const HTTP_VERSION: &str = "HTTP/1.1";

#[derive(Clap)]
#[clap(version = "1.0.0", author = "Jake Stanger <mail@jstanger.dev>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long, default_value = "7878")]
    pub port: u16,

    #[clap(short, long, default_value = "127.0.0.1")]
    pub host: String,

    #[clap(default_value = ".")]
    pub directory: String,
}

pub struct UploadFile<'a> {
    pub name: String,
    pub body: &'a[u8]
}