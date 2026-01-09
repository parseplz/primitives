use http::StatusCode;

#[derive(Debug, Default)]
pub struct ResponseLine {
    status: StatusCode,
}

impl ResponseLine {
    pub fn into_parts(self) -> StatusCode {
        self.status
    }
}
