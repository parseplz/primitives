use http::StatusCode;

#[derive(Debug, Default)]
pub struct ResponseLine {
    pub status: StatusCode,
}
