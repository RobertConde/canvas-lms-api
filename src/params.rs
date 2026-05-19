use serde_json::Value;

/// Recursively flatten a serde_json Value into Canvas bracket-notation query params.
///
/// `{"course": {"name": "Foo", "ids": [1, 2]}}` becomes:
/// `[("course[name]", "Foo"), ("course[ids][]", "1"), ("course[ids][]", "2")]`
pub fn to_canvas_params(key: &str, value: &Value) -> Vec<(String, String)> {
    match value {
        Value::Object(map) => map
            .iter()
            .flat_map(|(k, v)| to_canvas_params(&format!("{key}[{k}]"), v))
            .collect(),
        Value::Array(arr) => arr
            .iter()
            .flat_map(|v| to_canvas_params(&format!("{key}[]"), v))
            .collect(),
        Value::Bool(b) => vec![(key.to_string(), b.to_string())],
        Value::Number(n) => vec![(key.to_string(), n.to_string())],
        Value::String(s) => vec![(key.to_string(), s.clone())],
        Value::Null => vec![],
    }
}

/// Flatten a top-level Object value into Canvas query params.
pub fn flatten_params(value: &Value) -> Vec<(String, String)> {
    match value {
        Value::Object(map) => map
            .iter()
            .flat_map(|(k, v)| to_canvas_params(k, v))
            .collect(),
        _ => vec![],
    }
}

/// Wrap a serializable value under a bracket key and flatten.
/// e.g. wrap_params("course", &params) → [("course[name]", "Intro"), ...]
pub fn wrap_params<T: serde::Serialize>(key: &str, value: &T) -> Vec<(String, String)> {
    let json = serde_json::json!({ key: value });
    flatten_params(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flat_string() {
        let v = json!({"name": "Test"});
        assert_eq!(flatten_params(&v), vec![("name".into(), "Test".into())]);
    }

    #[test]
    fn nested_object() {
        let v = json!({"course": {"name": "Intro"}});
        assert_eq!(
            flatten_params(&v),
            vec![("course[name]".into(), "Intro".into())]
        );
    }

    #[test]
    fn array_values() {
        let v = json!({"ids": [1, 2, 3]});
        let params = flatten_params(&v);
        assert_eq!(params.len(), 3);
        assert!(params.iter().all(|(k, _)| k == "ids[]"));
        let vals: Vec<_> = params.iter().map(|(_, v)| v.as_str()).collect();
        assert_eq!(vals, ["1", "2", "3"]);
    }

    #[test]
    fn bool_lowercased() {
        let v = json!({"published": true});
        assert_eq!(flatten_params(&v), vec![("published".into(), "true".into())]);
    }

    #[test]
    fn null_omitted() {
        let v = json!({"name": null});
        assert!(flatten_params(&v).is_empty());
    }
}
