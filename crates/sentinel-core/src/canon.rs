use serde::Serialize;

/// v0.1 placeholder canonicalization:
/// - serialize to JSON Value
/// - recursively sort object keys
///
/// We will lock canonicalization properly (RFC 8785 JCS) before release.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut v = serde_json::to_value(value)?;
    sort_json_value(&mut v);
    serde_json::to_vec(&v)
}

fn sort_json_value(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            // Collect keys, sort, then rebuild the map in sorted key order.
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();

            let mut new_map = serde_json::Map::new();
            for k in keys {
                if let Some(mut val) = map.remove(&k) {
                    sort_json_value(&mut val);
                    new_map.insert(k, val);
                }
            }
            *map = new_map;
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sort_json_value(item);
            }
        }
        _ => {}
    }
}
