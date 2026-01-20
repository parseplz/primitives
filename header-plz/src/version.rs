pub const HTTP_0_9: &str = "HTTP/0.9";
pub const HTTP_1_0: &str = "HTTP/1.0";
pub const HTTP_1_1: &str = "HTTP/1.1";
pub const HTTP_2: &str = "HTTP/2";
pub const HTTP_3: &str = "HTTP/3";

pub enum Version {
    H09,
    H10,
    H11,
    H2,
    H3,
}

impl Version {
    pub fn as_str(&self) -> &str {
        use Version::*;
        match self {
            H09 => HTTP_0_9,
            H10 => HTTP_1_0,
            H11 => HTTP_1_1,
            H2 => HTTP_2,
            H3 => HTTP_3,
        }
    }

    pub fn for_request_line(&self) -> &str {
        use Version::*;
        match self {
            H09 => " HTTP/0.9\r\n",
            H10 => " HTTP/1.0\r\n",
            H11 => " HTTP/1.1\r\n",
            H2 => " HTTP/2\r\n",
            H3 => " HTTP/3\r\n",
        }
    }

    pub fn for_response_line(&self) -> &str {
        use Version::*;
        match self {
            H09 => "HTTP/0.9 ",
            H10 => "HTTP/1.0 ",
            H11 => "HTTP/1.1 ",
            H2 => "HTTP/2 ",
            H3 => "HTTP/3 ",
        }
    }
}
