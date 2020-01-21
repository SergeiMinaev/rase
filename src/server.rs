use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::string::String;
use log::{LevelFilter, info};
use chunked_transfer::Encoder;
use crate::ThreadPool;
use crate::config_parser;
use crate::logger;
use crate::mime;
use crate::http;
use crate::default_app::{default_app};


pub fn run_empty() {
    init_listener(default_app);
}

pub fn run(app: fn(request: &http::Request) -> String) {
    init_listener(app);
}

pub fn init_listener(app: fn(request: &http::Request) -> String) {
    log::set_logger(&logger::SIMPLE_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let conf = config_parser::get_config();

	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
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
                     app: fn(request: &http::Request) -> String) {
	let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    let request = http::parse_request(&request_str, &conf);
    info!("{} {} {}", request.host, request.method, request.url_path);

    if request.is_static {
        handle_static(stream, &request);
    } else {
        let response = app(&request);
        match stream.write(&response.into_bytes()) {
            Ok(_) => (),
            Err(e) => println!("Failed to send response: {}", e),
        };
        stream.flush().unwrap();
    }
}

fn handle_static(mut stream: TcpStream, request: &http::Request) {
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

    let mut encoded = Vec::new();
    {
        let mut encoder = Encoder::with_chunks_size(&mut encoded, 1024*1024);
        encoder.write_all(&buf).unwrap();
    }
    let f_len = f.metadata().unwrap().len();
    let content_len = format!("Content-Length: {}\r\n", f_len);

    let mime_line = match mime::get_mimetype(request.fs_path.as_str()) {
        None => String::from(""),
        Some(m) => format!("Content-Type: {}\r\n", m),
    };
    let headers = [
        "HTTP/1.1 200 OK\r\n",
        "Transfer-Encoding: chunked\r\n",
        content_len.as_str(),
        mime_line.as_str(),
        "\r\n"
    ];
    let mut response = headers.join("").to_string().into_bytes();
    response.extend(encoded);

    match stream.write(&response) {
        Ok(_) => (),
        Err(e) => println!("Failed sending response: {}", e),
    }
}
