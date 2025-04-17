use super::content_encoding::ContentEncoding;

#[derive(Debug, PartialEq, Copy, Clone, Default, Eq)]
pub enum TransferType {
    Chunked,
    ContentLength(usize),
    Close,
    #[default]
    Unknown,
}

// Convert content length to transfer type
pub fn cl_to_transfer_type(value: &str) -> TransferType {
    if let Ok(size) = value.parse::<usize>() {
        TransferType::ContentLength(size)
    } else {
        eprintln!("Failed to parse Content-Length| {}", value);
        TransferType::Close
    }
}

// Remove Chunked from transfer encoding,
pub fn parse_and_remove_chunked(value: &mut Option<Vec<ContentEncoding>>) -> Option<TransferType> {
    if let Some(vec) = value {
        let original_len = vec.len();
        vec.retain(|&ce| ce != ContentEncoding::Chunked);
        if vec.len() != original_len {
            if vec.is_empty() {
                *value = None;
            }
            return Some(TransferType::Chunked);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cl_to_transfer_type_val() {
        assert_eq!(TransferType::ContentLength(100), cl_to_transfer_type("100"));
    }

    #[test]
    fn test_cl_to_transfer_type_err() {
        assert_eq!(TransferType::Close, cl_to_transfer_type("test"));
    }

    #[test]
    fn test_cl_to_transfer_type_zero() {
        assert_eq!(TransferType::ContentLength(0), cl_to_transfer_type("0"));
    }

    #[test]
    fn test_parse_and_remove_chunked_only() {
        let vec = vec![ContentEncoding::Chunked];
        let mut arg = Some(vec);
        assert_eq!(
            Some(TransferType::Chunked),
            parse_and_remove_chunked(&mut arg)
        );
        assert!(arg.is_none());
    }

    #[test]
    fn test_parse_and_remove_chunked_empty() {
        assert_eq!(None, parse_and_remove_chunked(&mut Some(vec![])));
    }

    #[test]
    fn test_parse_and_remove_chunked_none() {
        assert_eq!(None, parse_and_remove_chunked(&mut None));
    }

    #[test]
    fn test_parse_and_remove_chunked_multiple() {
        let vec = vec![
            ContentEncoding::Brotli,
            ContentEncoding::Chunked,
            ContentEncoding::Compress,
            ContentEncoding::Deflate,
            ContentEncoding::Gzip,
            ContentEncoding::Identity,
            ContentEncoding::Zstd,
        ];
        let mut verify = vec.clone();
        verify.remove(1);

        let mut arg = Some(vec);
        assert_eq!(
            Some(TransferType::Chunked),
            parse_and_remove_chunked(&mut arg)
        );
        assert_eq!(arg.unwrap(), verify);
    }
}
