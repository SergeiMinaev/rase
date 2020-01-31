use crate::http::{Request, HttpResponse};

pub fn default_app(request: &Request) -> HttpResponse {
    let content: String = format!("<h1>Rase - rust (web) app server</h1>\
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
    let r = HttpResponse {
        code: 200,
        content: content,
    };
    return r;
}
