pub const CONNECT: &[u8] = b"CONNECT";
pub const DELETE: &[u8] = b"DELETE";
pub const GET: &[u8] = b"GET";
pub const HEAD: &[u8] = b"HEAD";
pub const OPTIONS: &[u8] = b"OPTIONS";
pub const PATCH: &[u8] = b"PATCH";
pub const POST: &[u8] = b"POST";
pub const PUT: &[u8] = b"PUT";
pub const TRACE: &[u8] = b"TRACE";

#[derive(PartialEq, Debug)]
pub enum Method {
    CONNECT,
    DELETE,
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
}

impl From<&[u8]> for Method {
    fn from(bytes: &[u8]) -> Method {
        match bytes {
            GET => Method::GET,
            CONNECT => Method::CONNECT,
            HEAD => Method::HEAD,
            OPTIONS => Method::OPTIONS,
            TRACE => Method::TRACE,
            POST => Method::POST,
            PUT => Method::PUT,
            PATCH => Method::PATCH,
            DELETE => Method::DELETE,
            _ => unreachable!("unknown method| {}", String::from_utf8_lossy(bytes)),
        }
    }
}

pub const METHODS_WITH_BODY: [Method; 4] =
    [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];
