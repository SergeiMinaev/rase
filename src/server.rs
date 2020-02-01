use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::string::String;
use log::{LevelFilter, info, error};
use libflate::gzip;
use crate::ThreadPool;
use crate::config_parser;
use crate::logger;
use crate::mime;
use crate::http;
use crate::default_app::{default_app};


pub fn run_empty() {
    init_listener(default_app);
}

pub fn run(app: fn(request: &http::Request) -> http::HttpResponse) {
    init_listener(app);
}

pub fn init_listener(app: fn(request: &http::Request) -> http::HttpResponse) {
    log::set_logger(&logger::SIMPLE_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let conf = config_parser::get_config();

	let listener = match TcpListener::bind(&conf.address_full) {
        Err(why) => {
            error!("{}", why);
            return;
        },
        Ok(listener) => listener,
    };
	let pool = ThreadPool::new(conf.thread_count);

	for stream in listener.incoming() {
        let conf = conf.clone();
		let stream = stream.unwrap();
		pool.execute(move || {
			handle_connection(stream, conf, app);
		});
	}
}

fn handle_connection(mut stream: TcpStream, conf: config_parser::Config,
                     app: fn(request: &http::Request) -> http::HttpResponse) {
	let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    let request = http::parse_request(&request_str, &conf);
    info!("{} {} {}", request.host, request.method, request.url_path);

    if request.is_static {
        handle_static(stream, &request);
    } else {
        let response = app(&request);
        let response_raw = http_response_to_str(&request, &response);

        match stream.write(&response_raw) {
            Ok(_) => (),
            Err(e) => println!("Failed to send a response: {}", e),
        };
        match stream.flush() {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        };
    }
}

fn http_response_to_str(request: &http::Request, r: &http::HttpResponse
                        ) ->  std::vec::Vec<u8> {
    let content: std::vec::Vec<u8>;
    if request.is_gzip_allowed {
        let mut encoder = gzip::Encoder::new(Vec::new()).unwrap();
        match encoder.write_all(&r.content.to_string().into_bytes()) {
            Ok(_) => (),
            Err(e) => error!("{}", e),
        };
        content = match encoder.finish().into_result() {
            Ok(d) => d,
            Err(_) => Vec::new(),
        };
    } else {
        content = r.content.as_bytes().to_vec();
    }
    let content_len = format!("Content-Length: {}\r\n", content.len());
    let mut resp = String::from(format!("HTTP/1.1 {} OK\r\nContent-Type: text/html\r\n", r.code));
    resp.push_str(content_len.as_str());
    if request.is_gzip_allowed {
        resp.push_str(&"Content-Encoding: gzip\r\n".to_string());
    }
    resp.push_str("\r\n");
    return [resp.into_bytes(), content].concat();
}

fn handle_static(mut stream: TcpStream, request: &http::Request) {
    let mut _tmp = [0; 512];
    stream.read(&mut _tmp).unwrap();

    let mut buf = Vec::new();
    let mut f = match File::open(&request.fs_path) {
        Ok(f) => f,
        Err(err) => {
            println!("Unable to open static file: {}", err);
            http::return_404(&stream);
            return;
        }
    };

    
    f.read_to_end(&mut buf).unwrap();

    let content_len = format!("Content-Length: {}\r\n", buf.len());

    let mime_line = match mime::get_mimetype(request.fs_path.as_str()) {
        None => String::from(""),
        Some(m) => format!("Content-Type: {}\r\n", m),
    };
    let headers = [
        "HTTP/1.1 200 OK\r\n",
        content_len.as_str(),
        mime_line.as_str(),
        "\r\n"
    ];
    let mut response = headers.join("").to_string().into_bytes();
    response.extend(buf);

    match stream.write_all(&response) {
        Ok(_) => (),
        Err(e) => println!("Failed sending response: {}", e),
    }
    match stream.flush() {
        Ok(_) => (),
        Err(e) => error!("{}", e),
    };
}
