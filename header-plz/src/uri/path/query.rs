use bytes::{BufMut, BytesMut};

use crate::abnf::{AMBER, EQUAL};

#[derive(Debug, PartialEq, Clone)]
pub struct KvPair {
    pub(crate) data: BytesMut,
    pub(crate) eq_index: Option<usize>,
    pub(crate) has_amber: bool,
}

impl KvPair {
    pub fn parse(data: BytesMut, has_amber: bool) -> KvPair {
        let eq_index = data.iter().position(|&c| c == EQUAL);
        KvPair {
            data,
            eq_index,
            has_amber,
        }
    }

    pub fn split_kv_pair(mut input: BytesMut) -> Vec<KvPair> {
        let mut pair = Vec::new();

        while !input.is_empty() {
            match input.iter().position(|&c| c == AMBER) {
                Some(index) => {
                    let data = input.split_to(index + 1);
                    pair.push(KvPair::parse(data, true));
                }
                None => {
                    // single pair
                    break;
                }
            }
        }

        if !input.is_empty() {
            pair.push(KvPair::parse(input, false));
        }
        pair
    }

    pub fn into_data(self) -> BytesMut {
        self.data
    }

    pub fn key(&self) -> Option<&[u8]> {
        self.eq_index.map(|idx| &self.data[..idx]).or_else(|| {
            Some(self.data.strip_suffix(&[AMBER]).unwrap_or(&self.data))
        })
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.eq_index.map(|index| {
            let val = &self.data[index + 1..];
            val.strip_suffix(&[AMBER]).unwrap_or(val)
        })
    }

    pub fn change_key(&mut self, key: &str) {
        let mut new_data = BytesMut::from(key);
        if let Some(index) = self.eq_index {
            new_data.extend_from_slice(&self.data[index..]);
        } else if self.has_amber {
            new_data.put_u8(AMBER);
        }
        *self = KvPair::parse(new_data, self.has_amber);
    }

    pub fn change_value(&mut self, value: &str) {
        if let Some(index) = self.eq_index {
            self.data.truncate(index + 1);
        } else {
            if self.has_amber {
                self.data.truncate(self.data.len() - 1);
            }
            self.data.put_u8(EQUAL);
            self.eq_index = Some(self.data.len() - 1);
        }
        self.data.extend_from_slice(value.as_bytes());
        if self.has_amber {
            self.data.put_u8(AMBER);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const KEY_IN: &[u8] = b"key";
    const KEY_OUT: &str = "yek";

    const VALUE_IN: &[u8] = b"value";
    const VALUE_OUT: &str = "eulav";
    const EMPTY: &[u8] = b"";
    const FINAL_KV: &str = "yek=eulav";

    #[rstest]
    // (input, init_key, init_val, new_key, exp_after_key, new_val, final)
    #[case("key", KEY_IN, None, KEY_OUT, "yek", VALUE_OUT, FINAL_KV)]
    #[case("key&", KEY_IN, None, KEY_OUT, "yek&", VALUE_OUT, "yek=eulav&")]
    #[case(
        "key=",
        KEY_IN,
        Some(EMPTY),
        KEY_OUT,
        "yek=",
        VALUE_OUT,
        "yek=eulav"
    )]
    #[case(
        "=value",
        EMPTY,
        Some(VALUE_IN),
        KEY_OUT,
        "yek=value",
        VALUE_OUT,
        "yek=eulav"
    )]
    #[case(
        "key=value",
        KEY_IN,
        Some(VALUE_IN),
        KEY_OUT,
        "yek=value",
        VALUE_OUT,
        "yek=eulav"
    )]
    #[case(
        "key=value&",
        KEY_IN,
        Some(VALUE_IN),
        KEY_OUT,
        "yek=value&",
        VALUE_OUT,
        "yek=eulav&"
    )]
    fn test_kv_pair_full_lifecycle_new(
        #[case] input: &str,
        #[case] exp_key: &[u8],
        #[case] exp_val: Option<&[u8]>,
        #[case] new_k: &str,
        #[case] exp_after_key: &str, // New parameter for intermediate state
        #[case] new_v: &str,
        #[case] final_buf: &str,
    ) {
        let mut pairs = KvPair::split_kv_pair(BytesMut::from(input));
        let mut kv = pairs.remove(0);

        // 1. Initial State Assertions
        assert_eq!(kv.key().unwrap(), exp_key);
        assert_eq!(kv.value(), exp_val);

        // 2. Change Key & Assert Intermediate State (The "change_key_only" check)
        kv.change_key(new_k);
        assert_eq!(kv.key().unwrap(), new_k.as_bytes());
        assert_eq!(
            kv.data,
            BytesMut::from(exp_after_key),
            "Data mismatch after changing key only"
        );

        // Verify internal state matches buffer content
        if exp_after_key.ends_with('&') {
            assert!(kv.has_amber, "has_amber should be true after key change");
        }
        if let Some(idx) = exp_after_key.find('=') {
            assert_eq!(
                kv.eq_index,
                Some(idx),
                "eq_index incorrect after key change"
            );
        } else {
            assert_eq!(
                kv.eq_index, None,
                "eq_index should be None if no '=' present"
            );
        }

        // 3. Change Value & Assert Final State
        kv.change_value(new_v);
        assert_eq!(kv.value().unwrap(), new_v.as_bytes());
        assert_eq!(
            kv.data,
            BytesMut::from(final_buf),
            "Data mismatch after changing value"
        );

        // Verify final internal state
        assert!(
            kv.eq_index.is_some(),
            "eq_index should be set after adding value"
        );
        if final_buf.ends_with('&') {
            assert!(kv.has_amber);
        }
    }

    #[rstest]
    // (input, changed_vec)
    #[case("key&key&key&key", vec!["yek=eulav&", "yek=eulav&", "yek=eulav&", "yek=eulav"])]
    #[case("key&key=value", vec!["yek=eulav&", "yek=eulav"])]
    #[case("key=&key=&key=&key", vec!["yek=eulav&", "yek=eulav&", "yek=eulav&", "yek=eulav"])]
    #[case("key=&key=value", vec!["yek=eulav&", "yek=eulav"])]
    #[case("=value&=value&=value&=value", vec!["yek=eulav&", "yek=eulav&", "yek=eulav&", "yek=eulav"])]
    #[case("=value&key=value", vec!["yek=eulav&", "yek=eulav"])]
    #[case("key=value&key=value", vec!["yek=eulav&", "yek=eulav"])]
    #[case("key&&key", vec!["yek=eulav&", "yek=eulav&", "yek=eulav"])]
    fn test_multiple_kv_full_lifecycle(
        #[case] input: &str,
        #[case] expected_vec: Vec<&str>,
    ) {
        let mut pairs = KvPair::split_kv_pair(BytesMut::from(input));
        assert_eq!(
            pairs.len(),
            expected_vec.len(),
            "Parsed pair count mismatch for input: {}",
            input
        );

        for (i, kv) in pairs.iter_mut().enumerate() {
            kv.change_key(KEY_OUT);
            kv.change_value(VALUE_OUT);

            assert_eq!(
                kv.data,
                BytesMut::from(expected_vec[i]),
                "Buffer mismatch at index {} for input: {}",
                i,
                input
            );
        }
    }

    #[rstest]
    #[case("a=b=c", &b"a"[..], Some(&b"b=c"[..]))]
    #[case("a%20b=c%20d", &b"a%20b"[..], Some(&b"c%20d"[..]))]
    #[case("ref=http://site.com", &b"ref"[..], Some(&b"http://site.com"[..]))]
    #[case("key-1.2_final=val", &b"key-1.2_final"[..], Some(&b"val"[..]))]
    fn test_kv_pair_edge_cases(
        #[case] input: &str,
        #[case] exp_key: &[u8],
        #[case] exp_val: Option<&[u8]>,
    ) {
        let mut pairs = KvPair::split_kv_pair(BytesMut::from(input));
        let mut kv = pairs.remove(0);

        assert_eq!(kv.key().unwrap(), exp_key);
        assert_eq!(kv.value(), exp_val);

        kv.change_key(KEY_OUT);
        kv.change_value(VALUE_OUT);
        assert_eq!(kv.data, BytesMut::from(FINAL_KV));
    }
}
