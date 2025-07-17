use crate::EQUAL;
use bytes::BytesMut;

#[derive(Default, Debug)]
pub struct QueryKvPair {
    pub key: Option<BytesMut>,
    equal: Option<BytesMut>,
    pub val: Option<BytesMut>,
    amp: Option<BytesMut>,
}

impl QueryKvPair {
    pub fn new(
        key: Option<BytesMut>,
        equal: Option<BytesMut>,
        val: Option<BytesMut>,
        amp: Option<BytesMut>,
    ) -> Self {
        QueryKvPair {
            key,
            equal,
            val,
            amp,
        }
    }

    pub fn parse(mut input: BytesMut, with_amper: bool) -> Option<Self> {
        if input.is_empty() {
            return None;
        }

        if let Some(index) = input.iter().position(|&c| c == EQUAL) {
            let raw_key = empty_to_none(input.split_to(index));
            let equal = Some(input.split_to(1)); // equal persent
            let amp = if with_amper {
                empty_to_none(input.split_off(input.len() - 1))
            } else {
                None
            };
            let val = empty_to_none(input);
            let kvpair = QueryKvPair::new(raw_key, equal, val, amp);
            Some(kvpair)
        } else {
            // only value
            let amp = if with_amper {
                empty_to_none(input.split_off(input.len() - 1))
            } else {
                None
            };
            let val = empty_to_none(input);
            let kvpair = QueryKvPair::new(None, None, val, amp);
            Some(kvpair)
        }
    }

    pub fn into_data(self) -> Option<BytesMut> {
        let mut data = self.key.unwrap_or_default();
        if let Some(equal) = self.equal {
            data.unsplit(equal);
        };
        if let Some(val) = self.val {
            data.unsplit(val);
        }
        if let Some(amp) = self.amp {
            data.unsplit(amp);
        }

        empty_to_none(data)
    }
}

pub fn empty_to_none(input: BytesMut) -> Option<BytesMut> {
    (!input.is_empty()).then_some(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_kv_pair_empty() {
        let query = QueryKvPair::parse(BytesMut::new(), false);
        assert!(query.is_none());
    }

    #[test]
    fn test_kv_pair() {
        let raw_query = BytesMut::from("a=b");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, false).unwrap();
        assert_eq!(query.key.as_ref().unwrap(), "a");
        assert_eq!(query.val.as_ref().unwrap(), "b");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_key_only() {
        let raw_query = BytesMut::from("a=");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, false).unwrap();
        assert_eq!(query.key.as_ref().unwrap(), "a");
        assert_eq!(query.val, None);
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_val_only() {
        let raw_query = BytesMut::from("=b");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, false).unwrap();
        assert_eq!(query.key, None);
        assert_eq!(query.val.as_ref().unwrap(), "b");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_no_equal() {
        let raw_query = BytesMut::from("a");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, false).unwrap();
        assert_eq!(query.key, None);
        assert_eq!(query.val.as_ref().unwrap(), "a");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_with_amp() {
        let raw_query = BytesMut::from("a=b&");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, true).unwrap();
        assert_eq!(query.key.as_ref().unwrap(), "a");
        assert_eq!(query.val.as_ref().unwrap(), "b");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_key_only_with_amp() {
        let raw_query = BytesMut::from("a=&");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, true).unwrap();
        assert_eq!(query.key.as_ref().unwrap(), "a");
        assert_eq!(query.val, None);
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_val_only_with_amp() {
        let raw_query = BytesMut::from("=b&");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, true).unwrap();
        assert_eq!(query.key, None);
        assert_eq!(query.val.as_ref().unwrap(), "b");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_no_equal_with_amp() {
        let raw_query = BytesMut::from("a&");
        let verify = raw_query.as_ptr_range();
        let query = QueryKvPair::parse(raw_query, true).unwrap();
        assert_eq!(query.key, None);
        assert_eq!(query.val.as_ref().unwrap(), "a");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_only_equal() {
        let raw = BytesMut::from("=");
        let verify = raw.as_ptr_range();
        let query = QueryKvPair::parse(raw, false).unwrap();
        assert!(query.key.is_none());
        assert!(query.val.is_none());
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }

    #[test]
    fn test_kv_pair_multiple_equals() {
        let raw = BytesMut::from("a=b=c");
        let verify = raw.as_ptr_range();
        let query = QueryKvPair::parse(raw, false).unwrap();
        assert_eq!(query.key.as_ref().unwrap(), "a");
        assert_eq!(query.val.as_ref().unwrap(), "b=c");
        assert_eq!(
            query
                .into_data()
                .unwrap()
                .as_ptr_range(),
            verify
        );
    }
}
