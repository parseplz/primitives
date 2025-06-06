use bytes::BytesMut;

// Trait to convert frame to bytesmut
pub trait Frame {
    fn into_bytes(self) -> BytesMut;
}
