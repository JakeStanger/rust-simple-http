use std::path::PathBuf;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Gets the path to a requested resource on disk
pub fn get_path(base_dir: &str, path: &str) -> PathBuf {
    PathBuf::from(base_dir).join(path.strip_prefix("/").unwrap_or(path))
}

/// Gets the sanitised filename, falling back to a random string
pub fn get_filename_or_fallback(name: Option<&&str>) -> String {
    name.map_or_else(|| random_string(6), |name| sanitise(name))
}

/// Generates a random alphanumeric string
pub fn random_string(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

/// Strips illegal or dangerous characters from filenames
pub fn sanitise(string: &str) -> String {
    string.replace("/", "_")
}

/// Interprets the given string
/// and checks if it looks like a path to a file.
///
/// This does not check if the path exists or perform any IO.
/// The check simply looks if the last segment contains an extension.
pub fn is_filepath(path: &PathBuf) -> bool {
    let name = path.file_name();
    let stem = path.file_stem();

    if let (Some(name), Some(stem)) = (name, stem) {
        name != stem
    } else {
        false
    }
}
