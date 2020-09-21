use minreq::{Request, URL};

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Safari/537.36";

pub fn get<T: Into<URL>>(url: T) -> Request {
    minreq::get(url).with_header("user-agent", UA)
}

pub fn post<T: Into<URL>>(url: T) -> Request {
    minreq::post(url).with_header("user-agent", UA)
}
