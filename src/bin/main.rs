use std::thread;
use std::fs::File;
use std::path::{Path};
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::net::TcpListener;
use chunked_transfer::Encoder;
use path_clean::{PathClean};
use rase::ThreadPool;
use rase::config_parser;


fn main() {
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

	let get = b"GET / HTTP/1.1\r\n";
	let sleep = b"GET /sleep HTTP/1.1\r\n";
    let s_static = String::from("GET static_url").replace("static_url",
                                    conf.static_url.as_str());

    if buffer.starts_with(s_static.as_bytes()) {
        let fname = &get_requested_path(&request_str);
        if !is_path_safe(fname) {
            println!("Path is not safe: {}", fname);
            return;
        }
        handle_static(stream, &fname, &conf);
    } else {
        let (status_code, filename) = if buffer.starts_with(get) {
            ("200 OK", "goodbye.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_millis(500));
            ("200 OK", "goodbye.html")
        } else {
            ("401 NOT FOUND", "goodbye.html")
        };

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let response = format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
                               status_code, contents.len(), contents);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn handle_static(mut stream: TcpStream, fname: &str, conf: &config_parser::Config) {
    let mut fullpath = String::from(conf.static_dir.as_str());
    fullpath.push_str(fname);
    let mut buf = Vec::new();
    let mut f = match File::open(fullpath.as_str()) {
        Ok(f) => f,
        Err(err) => {
            println!("Unable to open static file: {}", err);
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
    let content_len = format!("Content-Length: {}", f_len);
    let headers = [
        "HTTP/1.1 200 OK",
        "Transfer-Encoding: chunked",
        content_len.as_str(),
        "\r\n"
    ];
    let mut response = headers.join("\r\n")
        .to_string()
        .into_bytes();
    response.extend(encoded);

    match stream.write(&response) {
        Ok(_) => (),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn get_requested_path(request: &str) -> &str {
    let mut path: Vec<&str> = request.lines().next().unwrap().split("GET /static/")
        .collect();
    path = path[1].split(" ").collect();
    return path[0];
}

fn is_path_safe(requested_path: &str) -> bool {
    let conf = config_parser::get_config();
    let static_path = Path::new(&conf.static_dir);
    let full_path = static_path.join(requested_path).clean();
    return full_path.starts_with(static_path);
}
