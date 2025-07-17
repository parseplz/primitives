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
            CONNECT => Method::CONNECT,
            DELETE => Method::DELETE,
            GET => Method::GET,
            HEAD => Method::HEAD,
            OPTIONS => Method::OPTIONS,
            PATCH => Method::PATCH,
            POST => Method::POST,
            PUT => Method::PUT,
            TRACE => Method::TRACE,
            _ => unreachable!(
                "unknown method| {}",
                String::from_utf8_lossy(bytes)
            ),
        }
    }
}

pub const METHODS_WITH_BODY: [Method; 4] =
    [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_from_bytes() {
        assert_eq!(Method::from("CONNECT".as_bytes()), Method::CONNECT);
        assert_eq!(Method::from("DELETE".as_bytes()), Method::DELETE);
        assert_eq!(Method::from("GET".as_bytes()), Method::GET);
        assert_eq!(Method::from("HEAD".as_bytes()), Method::HEAD);
        assert_eq!(Method::from("OPTIONS".as_bytes()), Method::OPTIONS);
        assert_eq!(Method::from("PATCH".as_bytes()), Method::PATCH);
        assert_eq!(Method::from("POST".as_bytes()), Method::POST);
        assert_eq!(Method::from("PUT".as_bytes()), Method::PUT);
        assert_eq!(Method::from("TRACE".as_bytes()), Method::TRACE);
    }
}
