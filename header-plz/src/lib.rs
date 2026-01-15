#![allow(clippy::len_without_is_empty)]
pub mod abnf;
pub mod body_headers;
pub mod bytes_str;
pub mod const_headers;
pub mod error;
pub mod message_head;
pub mod methods;
pub mod status;
pub mod uri;

// control lines
use message_head::info_line;
// http1
pub use info_line::one::InfoLine as OneInfoLine;
pub use info_line::one::request::RequestLine as OneRequestLine;
pub use info_line::one::response::ResponseLine as OneResponseLine;

// http2
pub use info_line::two::request::RequestLine;
pub use info_line::two::response::ResponseLine;

// headers
pub use message_head::header_map::one::OneHeader;
pub use message_head::header_map::two::Header;

// headermap
pub use message_head::header_map::HeaderMap;
pub use message_head::header_map::OneHeaderMap;

// MessageHead
pub use message_head::MessageHead;
pub use message_head::OneMessageHead;

pub const HTTP_0_9: &str = "http/0.9";
pub const HTTP_1_0: &str = "http/1.0";
pub const HTTP_1_1: &str = "http/1.1";
pub const HTTP_2: &str = "http2";
pub const HTTP_3: &str = "http3";
