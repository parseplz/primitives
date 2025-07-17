use header_plz::body_headers::content_encoding::ContentEncoding;

// wiki - gzip -  1F 8B
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

/* wiki - deflate
78 01 	x␁ 	0 	zlib 	No Compression (no preset dictionary)
78 5E 	x^ 	Best speed (no preset dictionary)
78 9C 	xœ 	Default Compression (no preset dictionary)
78 DA 	xÚ 	Best Compression (no preset dictionary)
78 20 	x␠ 	No Compression (with preset dictionary)
78 7D 	x} 	Best speed (with preset dictionary)
78 BB 	x» 	Default Compression (with preset dictionary)
78 F9 	xù 	Best Compression (with preset dictionary)
*/
const DEFLATE_MAGIC_FIRST_BYTE: u8 = 0x78;
const DEFLATE_MAGIC_SECOND_BYTES: [u8; 8] =
    [0x01, 0x5E, 0x9C, 0xDA, 0x20, 0x7D, 0xBB, 0xF9];

// TODO: Zstandard Dictionary ?
// https://github.com/facebook/zstd/issues/768 - 0xFD2FB528
// wiki - zstd - 28 B5 2F FD
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xb5, 0x2f, 0xfd];

pub fn is_compressed(input: &[u8], encoding: &ContentEncoding) -> bool {
    match encoding {
        ContentEncoding::Deflate => {
            matches!(
                input,
                [first_byte, second_byte, ..]
                if *first_byte == DEFLATE_MAGIC_FIRST_BYTE
                    && DEFLATE_MAGIC_SECOND_BYTES.contains(second_byte)
            )
        }
        ContentEncoding::Gzip => input.starts_with(&GZIP_MAGIC),
        ContentEncoding::Zstd | ContentEncoding::Compress => {
            input.starts_with(&ZSTD_MAGIC)
        }
        ContentEncoding::Brotli
        | ContentEncoding::Compress
        | ContentEncoding::Identity => true,
        ContentEncoding::Chunked | ContentEncoding::Unknown(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use header_plz::body_headers::content_encoding::ContentEncoding;

    use crate::{
        decompression::{magic_bytes::is_compressed, single::tests::*},
        tests::*,
    };

    #[test]
    fn test_magic_bytes_deflate() {
        assert!(is_compressed(
            &compress_deflate(INPUT),
            &ContentEncoding::Deflate
        ));
    }

    #[test]
    fn test_magic_bytes_gzip() {
        assert!(is_compressed(&compress_gzip(INPUT), &ContentEncoding::Gzip));
    }

    #[test]
    fn test_magic_bytes_zstd() {
        assert!(is_compressed(&compress_zstd(INPUT), &ContentEncoding::Zstd));
    }

    #[test]
    fn test_magic_bytes_compress() {
        assert!(is_compressed(
            &compress_zstd(INPUT),
            &ContentEncoding::Compress
        ));
    }

    #[test]
    fn test_magic_bytes_identity() {
        assert!(is_compressed(INPUT, &ContentEncoding::Identity));
    }

    #[test]
    fn test_magic_bytes_chunked() {
        assert!(!is_compressed(INPUT, &ContentEncoding::Chunked));
    }
}
