use bytes::{BufMut, BytesMut};
use percent_encoding::percent_decode;

use crate::{
    abnf::{AMBER, FRAGMENT, QMARK},
    uri::{InvalidUri, path::PathAndQuery},
};

use super::query::KvPair;

/* uncomment for recursive decode
use percent_encoding::{
    AsciiSet, CONTROLS, percent_decode, percent_encode, utf8_percent_encode,
};
const QUERY_ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}');

const STRICT_ENCODE: &AsciiSet = &QUERY_ENCODE.add(b'&').add(b'=').add(b'+').add(b'%');
*/

#[derive(Default, PartialEq, Debug)]
pub struct PathAndQueryMut {
    path: BytesMut, // first "/" upto(including) "?"
    kvpair: Option<Vec<KvPair>>, // from "?" to end
                    // uncomment recursion
                    //depth: usize
}

impl PathAndQueryMut {
    pub fn parse(mut input: BytesMut) -> PathAndQueryMut {
        let mut uri = PathAndQueryMut::default();
        let mut query_idx = None;
        let mut fragment_idx = None;

        for (i, &b) in input.iter().enumerate() {
            if b == QMARK && query_idx.is_none() && fragment_idx.is_none() {
                query_idx = Some(i);
            } else if b == FRAGMENT && fragment_idx.is_none() {
                fragment_idx = Some(i);
                break;
            }
        }

        if let Some(index) = fragment_idx {
            input.truncate(index);
        }

        let mut depth = 0;
        if let Some(index) = query_idx {
            let mut raw_query = input.split_off(index + 1);
            loop {
                // empty
                if raw_query.is_empty() {
                    break;
                }

                // normal
                if raw_query.contains(&b'=') || raw_query.contains(&b'&') {
                    break;
                }

                let decoded = percent_decode(&raw_query).collect::<Vec<u8>>();

                // no change
                if decoded == raw_query.as_ref() {
                    break;
                }

                raw_query.clear();
                raw_query.extend_from_slice(&decoded);
                depth += 1;

                if depth >= 5 {
                    break;
                }
            }

            if !raw_query.is_empty() {
                uri.kvpair = Some(KvPair::split_kv_pair(raw_query));
                // uncomment recursion
                // uri.depth = depth;
            }
        }

        // 3. Path is what remains
        uri.path = input;
        uri
    }

    pub fn into_bytes(mut self) -> BytesMut {
        if let Some(kvpair) = self.kvpair {
            for data in kvpair.into_iter() {
                self.path.unsplit(data.into_data());
            }

            /* uncomment to add recursive encoding used by the original input
            if self.depth == 0 {
                for data in kvpair.into_iter() {
                    self.path.unsplit(data.into_data());
                }
            } else {
                let mut query_buf = BytesMut::new();
                for data in kvpair {
                    query_buf.unsplit(data.into_data());
                }
                for _ in 0..self.depth {
                    let mut buf = BytesMut::with_capacity(query_buf.len());
                    for chunk in percent_encode(&query_buf, STRICT_ENCODE) {
                        buf.extend_from_slice(chunk.as_bytes());
                    }
                    query_buf = buf
                }
                self.path.unsplit(query_buf);
            }
            */
        }
        self.path
    }

    pub fn path(&self) -> &[u8] {
        self.path.strip_suffix(&[QMARK]).unwrap_or(&self.path)
    }

    pub fn kv_iter_mut(&mut self) -> impl Iterator<Item = &mut KvPair> {
        self.kvpair.as_mut().into_iter().flatten()
    }

    pub fn insert(&mut self, to_insert: &str) {
        let to_insert =
            KvPair::parse(BytesMut::from(to_insert.as_bytes()), false);
        if let Some(kv) = self.kvpair.as_mut() {
            if let Some(last) = kv.last_mut()
                && !last.has_amber
            {
                last.data.put_u8(AMBER);
                last.has_amber = true;
            }
            kv.push(to_insert)
        } else {
            if !self.path.ends_with(&[QMARK]) {
                self.path.put_u8(QMARK);
            }
            self.kvpair = Some(vec![to_insert]);
        }
    }

    pub fn change_kv(&mut self, old_kv: &str, new_kv: &str) -> bool {
        let mut is_changed = false;
        let old_kv = KvPair::parse(BytesMut::from(old_kv), false);
        for kv in self.kv_iter_mut() {
            if *kv == old_kv {
                *kv = KvPair::parse(BytesMut::from(new_kv), false);
                is_changed = true;
            }
        }
        is_changed
    }

    pub fn change_key(&mut self, old_key: &str, new_key: &str) -> bool {
        let mut is_changed = false;
        self.kv_iter_mut()
            .filter(|kv| kv.key() == Some(old_key.as_bytes()))
            .for_each(|kv| {
                kv.change_key(new_key);
                is_changed = true;
            });
        is_changed
    }

    pub fn change_value(&mut self, old_value: &str, new_value: &str) -> bool {
        let mut is_changed = false;
        self.kv_iter_mut()
            .filter(|kv| kv.value() == Some(old_value.as_bytes()))
            .for_each(|kv| {
                kv.change_value(new_value);
                is_changed = true;
            });
        is_changed
    }
}

impl From<&PathAndQuery> for PathAndQueryMut {
    fn from(value: &PathAndQuery) -> Self {
        PathAndQueryMut::parse(BytesMut::from(value.data.as_ref()))
    }
}

impl TryFrom<PathAndQueryMut> for PathAndQuery {
    type Error = InvalidUri;

    fn try_from(value: PathAndQueryMut) -> Result<Self, Self::Error> {
        let buf = value.into_bytes();
        PathAndQuery::from_shared(buf.freeze())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_arguments)]

    use super::*;
    use bytes::BytesMut;
    use rstest::rstest;

    #[rstest]
    #[case("/path?a=b&c=d", "/path", "/path?a=b&c=d", 5)]
    #[case("/path", "/path", "/path", u16::MAX)]
    #[case("/", "/", "/", u16::MAX)]
    #[case("/?q=1", "/", "/?q=1", 1)]
    #[case("/a?b", "/a", "/a?b", 2)]
    fn test_from_path_mut_to_path(
        #[case] input: &str,
        #[case] expected_path: &str,
        #[case] expected_data: &str,
        #[case] expected_query_idx: u16,
    ) {
        let path_mut = PathAndQueryMut::parse(BytesMut::from(input));
        let path = PathAndQuery::try_from(path_mut).unwrap();

        assert_eq!(path.path(), expected_path);
        assert_eq!(path.data, expected_data.into());
        assert_eq!(path.query, expected_query_idx);
    }

    #[rstest]
    #[case(
        "/path?a=b&c=d", 
        b"/path", 
        vec![("a=b&", true), ("c=d", false)], 
        None
    )]
    #[case(
        "/path?key=val%20ue", 
        b"/path", 
        vec![("key=val%20ue", false)], 
        None
    )]
    #[case(
        "/path?k%20ey=value", 
        b"/path", 
        vec![("k%20ey=value", false)], 
        None
    )]
    #[case(
        "/?k%3Dv", 
        b"/", 
        vec![("k=v", false)], 
        Some("/?k=v")
    )]
    fn test_from_path_to_path_mut(
        #[case] input: &str,
        #[case] expected_path: &[u8],
        #[case] expected_pairs: Vec<(&str, bool)>,
        #[case] expected_output_bytes: Option<&str>,
    ) {
        let path = PathAndQuery::from_shared(bytes::Bytes::copy_from_slice(
            input.as_bytes(),
        ))
        .unwrap();
        let path_mut = PathAndQueryMut::from(&path);
        assert_eq!(path_mut.path(), expected_path);
        let expected_kv = expected_pairs
            .into_iter()
            .map(|(s, has_amber)| KvPair::parse(BytesMut::from(s), has_amber))
            .collect();
        assert_eq!(path_mut.kvpair, Some(expected_kv));
        let expected_bytes = expected_output_bytes.unwrap_or(input).as_bytes();
        assert_eq!(path_mut.into_bytes(), expected_bytes);
    }

    #[rstest]
    // case: (
    //   initial,
    //   to_insert,
    //   exp_after_insert,
    //   mod_k,
    //   exp_after_key_mod,
    //   mod_v,
    //   exp_after_val_mod,
    //   kv_to_revert,
    //   exp_final
    // )
    #[case(
        "/path",
        "key=val",
        "/path?key=val",
        "key2",
        "/path?key2=val",
        "val2",
        "/path?key2=val2",
        "key2=val2",
        "/path?key=val"
    )]
    #[case(
        "/path?existing=true",
        "key=val",
        "/path?existing=true&key=val",
        "key2",
        "/path?existing=true&key2=val",
        "val2",
        "/path?existing=true&key2=val2",
        "key2=val2",
        "/path?existing=true&key=val"
    )]
    #[case("/?", "a=b", "/?a=b", "c", "/?c=b", "d", "/?c=d", "c=d", "/?a=b")]
    #[case(
        "/#frag",
        "key=val",
        "/?key=val",
        "k2",
        "/?k2=val",
        "v2",
        "/?k2=v2",
        "k2=v2",
        "/?key=val"
    )]
    #[case(
        "/path",
        "key=",
        "/path?key=",
        "key2",
        "/path?key2=",
        "val",
        "/path?key2=val",
        "key2=val",
        "/path?key="
    )]
    #[case(
        "/path",
        "=val",
        "/path?=val",
        "key",
        "/path?key=val",
        "val2",
        "/path?key=val2",
        "key=val2",
        "/path?=val"
    )]
    #[case(
        "/?a=b&&c=d",
        "key=val",
        "/?a=b&&c=d&key=val",
        "k2",
        "/?a=b&&c=d&k2=val",
        "v2",
        "/?a=b&&c=d&k2=v2",
        "k2=v2",
        "/?a=b&&c=d&key=val"
    )]
    #[case(
        "/path?a=b&",
        "key=val",
        "/path?a=b&key=val",
        "k2",
        "/path?a=b&k2=val",
        "v2",
        "/path?a=b&k2=v2",
        "k2=v2",
        "/path?a=b&key=val"
    )]
    #[case(
        "/?query",
        "a=b",
        "/?query&a=b",
        "k",
        "/?query&k=b",
        "v",
        "/?query&k=v",
        "k=v",
        "/?query&a=b"
    )]
    #[case(
        "/?=value&key=&",
        "new=val",
        "/?=value&key=&new=val",
        "k",
        "/?=value&key=&k=val",
        "v",
        "/?=value&key=&k=v",
        "k=v",
        "/?=value&key=&new=val"
    )]
    #[case(
        "/?a=b#frag",
        "key=val",
        "/?a=b&key=val",
        "k2",
        "/?a=b&k2=val",
        "v2",
        "/?a=b&k2=v2",
        "k2=v2",
        "/?a=b&key=val"
    )]
    #[case(
        "/path?#frag",
        "key=val",
        "/path?key=val",
        "k2",
        "/path?k2=val",
        "v2",
        "/path?k2=v2",
        "k2=v2",
        "/path?key=val"
    )]
    #[case(
        "/?key=",
        "new=val",
        "/?key=&new=val",
        "k",
        "/?key=&k=val",
        "v",
        "/?key=&k=v",
        "k=v",
        "/?key=&new=val"
    )]
    #[case(
        "/?key=value",
        "new=val",
        "/?key=value&new=val",
        "k",
        "/?key=value&k=val",
        "v",
        "/?key=value&k=v",
        "k=v",
        "/?key=value&new=val"
    )]
    #[case(
        "/?a=b&c=d&e=f",
        "new=val",
        "/?a=b&c=d&e=f&new=val",
        "k",
        "/?a=b&c=d&e=f&k=val",
        "v",
        "/?a=b&c=d&e=f&k=v",
        "k=v",
        "/?a=b&c=d&e=f&new=val"
    )]
    #[case(
        "/?a=b&&c=d&",
        "key=val",
        "/?a=b&&c=d&key=val",
        "k",
        "/?a=b&&c=d&k=val",
        "v",
        "/?a=b&&c=d&k=v",
        "k=v",
        "/?a=b&&c=d&key=val"
    )]
    #[case(
        "/?=value&key=",
        "new=val",
        "/?=value&key=&new=val",
        "k",
        "/?=value&key=&k=val",
        "v",
        "/?=value&key=&k=v",
        "k=v",
        "/?=value&key=&new=val"
    )]
    #[case(
        "/?a=b&c=d#frag",
        "new=val",
        "/?a=b&c=d&new=val",
        "k",
        "/?a=b&c=d&k=val",
        "v",
        "/?a=b&c=d&k=v",
        "k=v",
        "/?a=b&c=d&new=val"
    )]
    fn test_uri(
        #[case] initial: &str,
        #[case] to_insert: &str,
        #[case] exp_after_insert: &str,
        #[case] mod_k: &str,
        #[case] exp_after_key_mod: &str,
        #[case] mod_v: &str,
        #[case] exp_after_val_mod: &str,
        #[case] kv_to_revert: &str,
        #[case] exp_final: &str,
    ) {
        let mut uri = PathAndQueryMut::parse(BytesMut::from(initial));
        uri.insert(to_insert);
        let bytes_1 = uri.into_bytes();
        assert_eq!(
            bytes_1,
            BytesMut::from(exp_after_insert),
            "Failed step 1: Insert"
        );

        let mut uri = PathAndQueryMut::parse(bytes_1);
        let (inserted_k, inserted_v) = to_insert.split_once('=').unwrap();

        let changed = uri.change_key(inserted_k, mod_k);
        assert!(
            changed,
            "Failed to change key from {} to {}",
            inserted_k, mod_k
        );

        let bytes_2 = uri.into_bytes();
        assert_eq!(
            bytes_2,
            BytesMut::from(exp_after_key_mod),
            "Failed step 2: Change Key"
        );

        let mut uri = PathAndQueryMut::parse(bytes_2);
        let changed = uri.change_value(inserted_v, mod_v);
        assert!(
            changed,
            "Failed to change value from {} to {}",
            inserted_v, mod_v
        );

        let bytes_3 = uri.into_bytes();
        assert_eq!(
            bytes_3,
            BytesMut::from(exp_after_val_mod),
            "Failed step 3: Change Value"
        );

        let mut uri = PathAndQueryMut::parse(bytes_3);
        let changed = uri.change_kv(kv_to_revert, to_insert);
        assert!(
            changed,
            "Failed step 4: Change KV from {} to {}",
            kv_to_revert, to_insert
        );

        let bytes_4 = uri.into_bytes();
        assert_eq!(
            bytes_4,
            BytesMut::from(exp_final),
            "Failed step 4: Final Verification"
        );
    }

    // tests for recursive encoding
    /*
    #[rstest]
    // depth 0
    #[case("/?k=v%201", "new", "/?k=v%201&new")]
    // depth 1
    // input: key=val (encoded) -> key%3Dval
    #[case("/?key%3Dval", "new", "/?key%3Dval%26new")]
    // depth 2
    // input: key=val (double encoded) -> key%253Dval
    #[case("/?key%253Dval", "new", "/?key%253Dval%2526new")]
    fn test_encoding_layers(#[case] input: &str, #[case] insert_key: &str, #[case] expected: &str) {
        let mut uri = Uri::parse(BytesMut::from(input));

        if input.contains("key") || input.contains("k") {
            assert!(uri.kv_iter_mut().count() > 0,);
        }

        uri.insert(insert_key);
        let output = uri.into_data();
        assert_eq!(output, BytesMut::from(expected));
    }
    */
}
