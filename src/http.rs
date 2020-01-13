use std::path::Path;
use path_clean::{PathClean};
use crate::config_parser::Config;

pub static RESPONSE_404: &'static [u8] = b"HTTP/1.1 404 Not Found\r\n\
        Content-Length: 22\r\n\
        Content-Type: text/html\r\n\r\n\
        <h1>404 Not found</h1>";

pub struct Request {
    pub version: String, 
    pub method: String,
    pub url_path: String,
    pub fs_path: String,
    pub is_gzip_allowed: bool,
    pub is_deflate_allowed: bool,
    pub is_static: bool,
}

fn get_default_request() -> Request {
    return Request {
        version: "".to_string(),
        method: "".to_string(),
        url_path: "".to_string(),
        fs_path: "".to_string(),
        is_gzip_allowed: false,
        is_deflate_allowed: false,
        is_static: false,
    }
}

fn get_fs_path(requested_path: &str, conf: &Config) -> String {
    let static_dir = Path::new(&conf.static_dir);
    let stripped_path = requested_path.replace(conf.static_url.as_str(), "");
    return static_dir.join(stripped_path).clean().to_str().unwrap().to_string();
}

fn is_path_safe(fs_path: &str, conf: &Config) -> bool {
    return fs_path.starts_with(&conf.static_dir);
}

pub fn parse_request<'a>(request_str: &'a str,
                         conf: &Config) -> Request{
    let mut request = get_default_request();
    for line in request_str.lines() {
        if line.starts_with("GET ") || line.starts_with("POST ") {
            let split: Vec<&'a str> = line.split(" ").collect();
            let count = line.split(" ").count();
            request.method = split[0].to_string();
            if count > 1 {
                request.url_path = split[1].to_string();
            }
            if count > 2 {
                request.version = split[2].to_string();
            }
        } else if line.starts_with("Accept-Encoding: ") {
            let split: Vec<&str> = line.split("Accept-Encoding: ").collect();
            let count = line.split(" ").count();
            if count > 1 {
                let encodings: Vec<&str> = split[1].split(", ").collect();
                for encoding in encodings {
                    if encoding == "gzip" {
                        request.is_gzip_allowed = true;
                    } else if encoding == "deflate" {
                        request.is_deflate_allowed = true;
                    }
                }
            }
        }
    }
    if request.url_path.starts_with(conf.static_url.as_str()) {
        request.fs_path = get_fs_path(&request.url_path, conf);
        if is_path_safe(&request.fs_path, conf) {
            request.is_static = true;
        }
    }
    return request;
}


