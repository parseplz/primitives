#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum TransferType {
    Close,
    ContentLength(usize),
    Chunked,
}

impl TransferType {
    // Convert content length to transfer type
    pub fn from_cl(value: &str) -> TransferType {
        if let Ok(size) = value.parse::<usize>() {
            TransferType::ContentLength(size)
        } else {
            eprintln!("Failed to parse Content-Length| {value}");
            TransferType::Close
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cl_to_transfer_type_val() {
        assert_eq!(
            TransferType::ContentLength(100),
            TransferType::from_cl("100")
        );
    }

    #[test]
    fn test_cl_to_transfer_type_err() {
        assert_eq!(TransferType::Close, TransferType::from_cl("test"));
    }

    #[test]
    fn test_cl_to_transfer_type_zero() {
        assert_eq!(TransferType::ContentLength(0), TransferType::from_cl("0"));
    }
}
