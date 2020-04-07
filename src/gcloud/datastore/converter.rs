use super::ResponseError;
use serde::de::DeserializeOwned;
use serde_json::map::Map;
use serde_json::{Number, Value};

pub fn convert_result<D>(v: &Value) -> Result<D, ResponseError>
where
    D: DeserializeOwned,
{
    match v.get("found") {
        Some(found) => {
            let prop_map = found
                .get(0)
                .unwrap()
                .get("entity")
                .unwrap()
                .get("properties")
                .unwrap();

            let v = to_object(prop_map);
            Ok(serde_json::from_value(v).unwrap())
        }
        None => Err(ResponseError::new_internal_server_error(
            "blub".to_string(),
            "error read response lookup body",
        )),
    }

    // if let Some(missing) = v.get("missing") {
    //     Err(ResponseError::new_internal_server_error(
    //         "".to_string(),
    //         "error read response lookup body",
    //     ))
    // }
}

// example:
// "Name": {"stringValue": "its me"}
// attr_name (attr): { datatype (dt) : value (v) }
pub fn to_object(map: &Value) -> Value {
    let mut result_map = Map::new();
    for (attr, dt_v) in map.as_object().unwrap() {
        for (dt, v) in dt_v.as_object().unwrap() {
            let to_val = to_value(dt, v.as_str().unwrap());
            result_map.insert(attr.to_string(), to_val);
        }
    }
    Value::Object(result_map)
}

// still missing datatypes:
// https://cloud.google.com/datastore/docs/reference/data/rest/v1/projects/runQuery#Value
//
// convert: "integerValue": "42" (datatype = "integerValue", val = "42") -> Value::Number(42)
pub fn to_value(datatype: &str, val: &str) -> Value {
    match datatype {
        "nullValue" => Value::Null,
        "doubleValue" => {
            let v = val.parse().unwrap();
            let n = Number::from_f64(v).unwrap();
            Value::Number(n)
        }
        "integerValue" => {
            let v: isize = val.parse().unwrap();
            Value::Number(Number::from(v))
        }
        "booleanValue" => {
            let v = val.parse().unwrap();
            Value::Bool(v)
        }
        // timestampValue | stringValue
        _ => {
            let v = val.to_string();
            Value::String(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_value() {
        assert_eq!(
            Value::Number(Number::from(42)),
            to_value("integerValue", "42")
        );

        assert_eq!(Value::Null, to_value("nullValue", "null"));

        assert_eq!(
            Value::String("foo".to_string()),
            to_value("stringValue", "foo")
        );
    }

    #[test]
    fn test_to_object() {
        let json = r#"{
            "HeroID": {"integerValue": "42"},
            "Action": {"stringValue": "List"}
          }"#;
        let value_map: Value = serde_json::from_str(json).unwrap();
        let result = to_object(&value_map);

        let mut map = Map::new();
        map.insert(String::from("HeroID"), Value::Number(Number::from(42)));
        map.insert(String::from("Action"), Value::String("List".to_string()));
        assert_eq!(Value::Object(map), result);
    }
}
