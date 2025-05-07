use bytes::BytesMut;

use crate::{FRAGMENT, QMARK, query::Query};

/*

abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
|-|   |-------------------------------||--------| |-------------------| |-----|
 |                  |                       |               |              |
scheme          authority                 path            query         fragment

*/

#[derive(Default)]
pub struct Uri {
    path: BytesMut,             // first "/" upto "?" or "#" or end, always present
    query: Option<Query>,       // first "?" upto "#" or end
    fragment: Option<BytesMut>, // from # to end - "/" and "?" are allowed
}

impl Uri {
    fn parse(mut input: BytesMut) -> Uri {
        let mut uri = Uri::default();

        // fragment
        if let Some(index) = input.iter().position(|&c| c == FRAGMENT) {
            uri.fragment = Some(input.split_off(index));
        }

        // query
        if let Some(index) = input.iter().position(|&c| c == QMARK) {
            let raw_query = input.split_off(index);
            uri.query = Query::parse(raw_query);
        }

        // path
        uri.path = input;

        uri
    }

    fn into_data(mut self) -> BytesMut {
        if let Some(query_data) = self.query.and_then(Query::into_data) {
            self.path.unsplit(query_data);
        }
        if let Some(fragment) = self.fragment {
            self.path.unsplit(fragment);
        }
        self.path
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_uri_parse_simple() {
        let buf = BytesMut::from("/path");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/path");
    }

    #[test]
    fn test_uri_parse_fragment_only() {
        let buf = BytesMut::from("/#fragment");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        assert_eq!(uri.fragment, Some("#fragment".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_no_key() {
        let buf = BytesMut::from("/?query");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, None);
        assert_eq!(kv_pair[0].val, Some("query".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_key_only() {
        let buf = BytesMut::from("/?key=");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("key".into()));
        assert_eq!(kv_pair[0].val, None);
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_key_and_value() {
        let buf = BytesMut::from("/?key=value");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("key".into()));
        assert_eq!(kv_pair[0].val, Some("value".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_multiple() {
        let buf = BytesMut::from("/?a=b&c=d&e=f");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(kv_pair[1].key, Some("c".into()));
        assert_eq!(kv_pair[1].val, Some("d".into()));
        assert_eq!(kv_pair[2].key, Some("e".into()));
        assert_eq!(kv_pair[2].val, Some("f".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_trailing_ampersand() {
        let buf = BytesMut::from("/?a=b&");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_double_ampersand() {
        let buf = BytesMut::from("/?a=b&&c=d");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(kv_pair[1].key, None);
        assert_eq!(kv_pair[1].val, None);
        assert_eq!(kv_pair[2].key, Some("c".into()));
        assert_eq!(kv_pair[2].val, Some("d".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_double_ampersand_and_trailing_ampersand() {
        let buf = BytesMut::from("/?a=b&&c=d&");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(kv_pair[1].key, None);
        assert_eq!(kv_pair[1].val, None);
        assert_eq!(kv_pair[2].key, Some("c".into()));
        assert_eq!(kv_pair[2].val, Some("d".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_empty_key_and_value() {
        let buf = BytesMut::from("/?=value&key=");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, None);
        assert_eq!(kv_pair[0].val, Some("value".into()));
        assert_eq!(kv_pair[1].key, Some("key".into()));
        assert_eq!(kv_pair[1].val, None);
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_only_with_empty_key_and_value_and_trailing_ampersand() {
        let buf = BytesMut::from("/?=value&key=&");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, None);
        assert_eq!(kv_pair[0].val, Some("value".into()));
        assert_eq!(kv_pair[1].key, Some("key".into()));
        assert_eq!(kv_pair[1].val, None);
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_query_with_fragments() {
        let buf = BytesMut::from("/?a=b#frag");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }

    #[test]
    fn test_uri_parse_multiple_queries_with_fragments() {
        let buf = BytesMut::from("/?a=b&c=d#frag");
        let verify = buf.as_ptr_range();
        let uri = Uri::parse(buf);
        assert_eq!(uri.path, "/");
        let kv_pair = uri.query.as_ref().unwrap().kv_pair.as_ref().unwrap();
        assert_eq!(kv_pair[0].key, Some("a".into()));
        assert_eq!(kv_pair[0].val, Some("b".into()));
        assert_eq!(kv_pair[1].key, Some("c".into()));
        assert_eq!(kv_pair[1].val, Some("d".into()));
        assert_eq!(uri.into_data().as_ptr_range(), verify);
    }
}
