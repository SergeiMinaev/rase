use std::net::TcpListener;
use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use rase::ThreadPool;
use rase::config_parser;


fn main() {
    let conf = config_parser::get_config();
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
	let pool = ThreadPool::new(conf.thread_count);

	for stream in listener.incoming() {
        //println!("stream incoming...");
		let stream = stream.unwrap();
        //thread::spawn(|| {
		pool.execute(|| {
			handle_connection(stream);
		});
	}
}

fn handle_connection(mut stream: TcpStream) {

	let mut buffer = [0; 512];

	stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

	let get = b"GET / HTTP/1.1\r\n";
	let sleep = b"GET /sleep HTTP/1.1\r\n";

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
