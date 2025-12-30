use bytes::Bytes;

pub const CONNECT: &[u8] = b"CONNECT";
pub const DELETE: &[u8] = b"DELETE";
pub const GET: &[u8] = b"GET";
pub const HEAD: &[u8] = b"HEAD";
pub const OPTIONS: &[u8] = b"OPTIONS";
pub const PATCH: &[u8] = b"PATCH";
pub const POST: &[u8] = b"POST";
pub const PUT: &[u8] = b"PUT";
pub const TRACE: &[u8] = b"TRACE";

#[derive(PartialEq, Debug, Default)]
pub enum Method {
    CONNECT,
    DELETE,
    #[default]
    GET,
    HEAD,
    OPTIONS,
    PATCH,
    POST,
    PUT,
    TRACE,
    UNKNOWN(Bytes),
}

impl Method {
    fn unknown(src: &[u8]) -> Self {
        Self::UNKNOWN(Bytes::from_owner(src.to_owned()))
    }
}

impl From<&[u8]> for Method {
    fn from(src: &[u8]) -> Method {
        match src.len() {
            0 => todo!(),
            3 => match src {
                GET => Method::GET,
                PUT => Method::PUT,
                _ => Method::unknown(src),
            },
            4 => match src {
                HEAD => Method::HEAD,
                POST => Method::POST,
                _ => Method::unknown(src),
            },
            5 => match src {
                PATCH => Method::PATCH,
                TRACE => Method::TRACE,
                _ => Method::unknown(src),
            },

            6 => match src {
                DELETE => Method::DELETE,
                _ => Method::unknown(src),
            },
            7 => match src {
                CONNECT => Method::CONNECT,
                OPTIONS => Method::OPTIONS,
                _ => Method::unknown(src),
            },
            _ => Method::unknown(src),
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
