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
        Value::Number(v) => write_number(v, out)?,
        Value::String(v) => {
            let encoded = serde_json::to_string(v).expect("serde_json string encoding is infallible");
            out.push_str(&encoded);
        }
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
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

fn write_number(value: &serde_json::Number, out: &mut String) -> Result<(), CanonicalError> {
    if let Some(integer) = value.as_i64() {
        out.push_str(&integer.to_string());
        return Ok(());
    }

    if let Some(integer) = value.as_u64() {
        out.push_str(&integer.to_string());
        return Ok(());
    }

    if let Some(float) = value.as_f64() {
        if !float.is_finite() {
            return Err(CanonicalError::NonFiniteNumber);
        }

        // Normalize finite integral IEEE-754 values to the integer form when they
        // are exactly representable. This removes lexical differences such as
        // 1.0 vs 1 while keeping large floating-point values in serde_json's
        // deterministic shortest representation.
        if float.fract() == 0.0 && float.abs() <= 9_007_199_254_740_991.0 {
            out.push_str(&(float as i64).to_string());
        } else {
            out.push_str(&float.to_string());
        }
        return Ok(());
    }

    Err(CanonicalError::NonFiniteNumber)
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

    #[test]
    fn numeric_lexical_forms_share_a_canonical_integer() {
        let a: Value = serde_json::from_str("1").unwrap();
        let b: Value = serde_json::from_str("1.0").unwrap();
        let c: Value = serde_json::from_str("1e0").unwrap();
        assert_eq!(canonicalize(&a).unwrap(), "1");
        assert_eq!(canonicalize(&b).unwrap(), "1");
        assert_eq!(canonicalize(&c).unwrap(), "1");
        assert_eq!(hash(&a).unwrap(), hash(&b).unwrap());
        assert_eq!(hash(&b).unwrap(), hash(&c).unwrap());
    }

    #[test]
    fn numeric_vectors_are_stable() {
        let vectors = [
            ("1", "1"),
            ("1.0", "1"),
            ("1e0", "1"),
            ("-0", "0"),
            ("-42", "-42"),
            ("9007199254740991", "9007199254740991"),
        ];

        for (input, expected) in vectors {
            let value: Value = serde_json::from_str(input).unwrap();
            assert_eq!(canonicalize(&value).unwrap(), expected, "vector {input}");
        }
    }

    #[test]
    fn non_finite_numbers_are_rejected_when_constructed() {
        let value = serde_json::Number::from_f64(f64::NAN);
        assert!(value.is_none());
    }
}
