use bytes::BytesMut;

use crate::{AMBER, querykvpair::QueryKvPair};

#[derive(Default)]
pub struct Query {
    qmark: BytesMut, //  "?" mandatory as identifies query
    pub kv_pair: Option<Vec<QueryKvPair>>,
}

impl Query {
    fn new(qmark: BytesMut, kv_pair: Option<Vec<QueryKvPair>>) -> Self {
        Query {
            qmark,
            kv_pair,
        }
    }

    pub fn parse(mut input: BytesMut) -> Option<Self> {
        let qmark = input.split_to(1); // always present
        let kv_pair = split_query_kv_pair(input);
        Some(Query::new(qmark, kv_pair))
    }

    pub fn into_data(self) -> Option<BytesMut> {
        let mut data = self.qmark;
        if let Some(kv_vec) = self.kv_pair {
            for kv in kv_vec {
                if let Some(kv_data) = kv.into_data() {
                    data.unsplit(kv_data);
                }
            }
        }
        Some(data)
    }
}

fn split_query_kv_pair(mut input: BytesMut) -> Option<Vec<QueryKvPair>> {
    let mut kv_vec = Vec::new();

    while !input.is_empty() {
        match input.iter().position(|&c| c == AMBER) {
            Some(index) => {
                let kv = input.split_to(index + 1);
                if let Some(v) = QueryKvPair::parse(kv, true) {
                    kv_vec.push(v)
                }
            }
            None => {
                // single pair
                if let Some(v) = QueryKvPair::parse(input, false) {
                    kv_vec.push(v)
                }
                break;
            }
        }
    }

    (!kv_vec.is_empty()).then_some(kv_vec)
}

mod tests {
    use super::*;

    #[test]
    fn test_query_parse_only_qmark() {
        let buf = BytesMut::from("?");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        assert_eq!(query.qmark, "?");
        assert!(query.kv_pair.is_none());
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_parse_single() {
        let buf = BytesMut::from("?a=b");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        assert_eq!(query.qmark, "?");
        assert_eq!(query.qmark, "?");
        let kv_vec = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv_vec[0].key, Some("a".into()));
        assert_eq!(kv_vec[0].val, Some("b".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_parse_multiple() {
        let buf = BytesMut::from("?a=b&c=d&e=f");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        assert_eq!(query.qmark, "?");
        let kv_vec = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv_vec[0].key, Some("a".into()));
        assert_eq!(kv_vec[0].val, Some("b".into()));
        assert_eq!(kv_vec[1].key, Some("c".into()));
        assert_eq!(kv_vec[1].val, Some("d".into()));
        assert_eq!(kv_vec[2].key, Some("e".into()));
        assert_eq!(kv_vec[2].val, Some("f".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_parse_empty_key_and_value() {
        let buf = BytesMut::from("?=value&key=");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        let kv = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv[0].key, None);
        assert_eq!(kv[0].val, Some("value".into()));
        assert_eq!(kv[1].key, Some("key".into()));
        assert_eq!(kv[1].val, None);
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_parse_key_only() {
        let buf = BytesMut::from("?key");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        let kv = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv[0].key, None);
        assert_eq!(kv[0].val, Some("key".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_parse_trailing_ampersand() {
        let buf = BytesMut::from("?a=b&");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        assert_eq!(query.qmark, "?");
        let kv_vec = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv_vec[0].key, Some("a".into()));
        assert_eq!(kv_vec[0].val, Some("b".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_double_ampersand() {
        let buf = BytesMut::from("?a=b&&c=d");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        let kv = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv[0].key, Some("a".into()));
        assert_eq!(kv[0].val, Some("b".into()));
        assert_eq!(kv[1].key, None); // empty pair
        assert_eq!(kv[1].val, None);
        assert_eq!(kv[2].key, Some("c".into()));
        assert_eq!(kv[2].val, Some("d".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }

    #[test]
    fn test_query_only_value() {
        let buf = BytesMut::from("?=value");
        let verify = buf.as_ptr_range();
        let query = Query::parse(buf).unwrap();
        let kv = query.kv_pair.as_ref().unwrap();
        assert_eq!(kv[0].key, None);
        assert_eq!(kv[0].val, Some("value".into()));
        assert_eq!(query.into_data().unwrap().as_ptr_range(), verify);
    }
}
