pub const HTTP_0_9: &str = "HTTP/0.9";
pub const HTTP_1_0: &str = "HTTP/1.0";
pub const HTTP_1_1: &str = "HTTP/1.1";
pub const HTTP_2: &str = "HTTP/2";
pub const HTTP_3: &str = "HTTP/3";

#[derive(Clone, Default, Eq, Debug, PartialEq)]
pub enum Version {
    H09,
    H10,
    #[default]
    H11,
    H2,
    H3,
}

impl Version {
    pub fn as_str(&self) -> &str {
        use Version::*;
        match self {
            H11 => HTTP_1_1,
            H2 => HTTP_2,
            H09 => HTTP_0_9,
            H10 => HTTP_1_0,
            H3 => HTTP_3,
        }
    }

    pub fn for_request_line(&self) -> &str {
        use Version::*;
        match self {
            H11 => " HTTP/1.1\r\n",
            H2 => " HTTP/2\r\n",
            H09 => " HTTP/0.9\r\n",
            H10 => " HTTP/1.0\r\n",
            H3 => " HTTP/3\r\n",
        }
    }

    pub fn for_response_line(&self) -> &str {
        use Version::*;
        match self {
            H11 => "HTTP/1.1 ",
            H2 => "HTTP/2 ",
            H09 => "HTTP/0.9 ",
            H10 => "HTTP/1.0 ",
            H3 => "HTTP/3 ",
        }
    }

    pub fn parse_request_version(input: &[u8]) -> Option<Version> {
        use Version::*;
        match input {
            b" HTTP/1.1\r\n" => Some(H11),
            b" HTTP/2\r\n" => Some(H2),
            b" HTTP/0.9\r\n" => Some(H09),
            b" HTTP/1.0\r\n" => Some(H10),
            b" HTTP/3\r\n" => Some(H3),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::Version;

    #[rstest]
    #[case(b" HTTP/0.9\r\n", Some(Version::H09))]
    #[case(b" HTTP/1.0\r\n", Some(Version::H10))]
    #[case(b" HTTP/1.1\r\n", Some(Version::H11))]
    #[case(b" HTTP/2\r\n", Some(Version::H2))]
    #[case(b" HTTP/3\r\n", Some(Version::H3))]
    fn test_valid_http_versions(
        #[case] input: &[u8],
        #[case] expected: Option<Version>,
    ) {
        assert_eq!(Version::parse_request_version(input), expected);
    }
}
