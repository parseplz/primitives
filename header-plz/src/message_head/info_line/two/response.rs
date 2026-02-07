use crate::status::StatusCode;

#[derive(Clone, Debug, PartialEq)]
pub struct ResponseLine {
    status: StatusCode,
}

impl Default for ResponseLine {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
        }
    }
}

impl ResponseLine {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
        }
    }

    pub fn into_parts(self) -> StatusCode {
        self.status
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status
    }

    pub fn status(&self) -> &StatusCode {
        &self.status
    }
}
