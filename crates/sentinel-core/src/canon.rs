use serde::Serialize;

/// RFC 8785 JSON Canonicalization Scheme (JCS)
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    serde_jcs::to_vec(&value)
}
