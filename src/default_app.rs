use crate::http::{Request};

pub fn default_app(request: &Request) -> String {
    let content = format!("<h1>Rase - rust (web) app server</h1>\
                        <h3>Request:</h3>\
                        version: {}<br>\
                        host: {}<br>\
                        method: {}<br>\
                        url_path: {}<br>\
                        ",
                        request.version,
                        request.host,
                        request.method,
                        request.url_path);
    let content_len = format!("Content-Length: {}\r\n", content.len());
    let mut resp = String::from("HTTP/1.1 200 OK\r\n\
                    Content-Type: text/html\r\n");
    resp.push_str(content_len.as_str());
    resp.push_str("\r\n\r\n");
    resp.push_str(content.as_str());
    return resp;
}
