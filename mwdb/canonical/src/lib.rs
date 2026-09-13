use blake3::Hasher;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalError {
    #[error("non-finite number is not supported")]
    NonFiniteNumber,
}

pub fn canonicalize(value: &Value) -> Result<String, CanonicalError> {
    let mut out = String::new();
    write_value(value, &mut out)?;
    Ok(out)
}

pub fn hash(value: &Value) -> Result<[u8; 32], CanonicalError> {
    let encoded = canonicalize(value)?;
    let mut hasher = Hasher::new();
    hasher.update(encoded.as_bytes());
    Ok(*hasher.finalize().as_bytes())
}

fn write_value(value: &Value, out: &mut String) -> Result<(), CanonicalError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(v) => out.push_str(if *v { "true" } else { "false" }),
        Value::Number(v) => {
            out.push_str(&v.to_string());
        }
        Value::String(v) => {
            let encoded = serde_json::to_string(v).expect("serde_json string encoding is infallible");
            out.push_str(&encoded);
        }
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 { out.push(','); }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index != 0 { out.push(','); }
                let encoded = serde_json::to_string(*key).expect("serde_json string encoding is infallible");
                out.push_str(&encoded);
                out.push(':');
                write_value(map.get(*key).expect("sorted key came from map"), out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_key_order_is_canonical() {
        let a = serde_json::json!({"b":2,"a":1});
        let b = serde_json::json!({"a":1,"b":2});
        assert_eq!(canonicalize(&a).unwrap(), canonicalize(&b).unwrap());
        assert_eq!(hash(&a).unwrap(), hash(&b).unwrap());
    }

    #[test]
    fn array_order_is_semantic() {
        let a = serde_json::json!([1,2]);
        let b = serde_json::json!([2,1]);
        assert_ne!(canonicalize(&a).unwrap(), canonicalize(&b).unwrap());
    }

    #[test]
    fn nested_objects_are_sorted_recursively() {
        let value = serde_json::json!({"z":{"b":2,"a":1},"a":[{"d":4,"c":3}]});
        assert_eq!(canonicalize(&value).unwrap(), r#"{"a":[{"c":3,"d":4}],"z":{"a":1,"b":2}}"#);
    }
}
