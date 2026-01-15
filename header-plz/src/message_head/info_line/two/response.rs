use crate::status::StatusCode;

#[derive(Debug)]
pub struct ResponseLine {
    status: StatusCode,
}

impl Default for ResponseLine {
    fn default() -> Self {
        Self {
            status: StatusCode::from_u16(200).unwrap(),
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
