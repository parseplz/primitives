use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::ContentType;
use crate::mime_type::*;

lazy_static! {
    pub static ref EXTENSION_MAP: HashMap<&'static str, ContentType> = {
        let mut m = HashMap::new();
        for (ct, exts) in [
            (ContentType::Application, &EXT_APP[..]),
            (ContentType::Audio, &EXT_AUDIO[..]),
            (ContentType::Font, &EXT_FONT[..]),
            (ContentType::Image, &EXT_IMAGE[..]),
            (ContentType::Message, &EXT_MESSAGE[..]),
            (ContentType::Model, &EXT_MODEL[..]),
            (ContentType::Text, &EXT_TEXT[..]),
            (ContentType::Video, &EXT_VIDEO[..]),
        ]
        .iter()
        {
            for &ext in *exts {
                m.insert(ext, *ct);
            }
        }
        m
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_content_type() {
        assert_eq!(
            EXTENSION_MAP
                .get("png")
                .copied()
                .unwrap(),
            ContentType::Image
        );

        assert_eq!(
            EXTENSION_MAP
                .get("mp4")
                .copied()
                .unwrap(),
            ContentType::Video
        );

        assert!(EXTENSION_MAP.get("").copied().is_none());
    }
}
