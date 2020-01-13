use std::thread;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::net::TcpListener;
use log::{LevelFilter, info};
use chunked_transfer::Encoder;
use rase::ThreadPool;
use rase::config_parser;
use rase::logger;
use rase::mime;
use rase::http;


pub fn main() {
    log::set_logger(&logger::SIMPLE_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let conf = config_parser::get_config();

	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
	let pool = ThreadPool::new(conf.thread_count);

	for stream in listener.incoming() {
        let conf = conf.clone();
		let stream = stream.unwrap();
		pool.execute(move || {
			handle_connection(stream, conf);
		});
	}
}

fn handle_connection(mut stream: TcpStream, conf: config_parser::Config) {
	let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();
    let request_str = String::from_utf8_lossy(&buffer);
    let request = http::parse_request(&request_str, &conf);
    info!("{} {} {}", request.host, request.method, request.url_path);

	let get = b"GET / HTTP/1.1\r\n";
	let sleep = b"GET /sleep HTTP/1.1\r\n";

    if request.is_static {
        handle_static(stream, &request);
    } else {
        let (status_code, filename) = if buffer.starts_with(get) {
            ("200 OK", "goodbye.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_millis(500));
            ("200 OK", "goodbye.html")
        } else {
            ("404", "")
        };
        if status_code == "404" {
            http::return_404(&stream);
            return;
        }

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let response = format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
                               status_code, contents.len(), contents);

        stream.write(response.as_bytes()).unwrap();
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
