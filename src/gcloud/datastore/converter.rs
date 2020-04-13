use super::{Entity, Error};
use http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::map::Map;
use serde_json::{Number, Value};

pub fn deserialize_lookup_result<D>(v: &Value) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    if let Some(found) = v.get("found") {
        let prop_map = found
            .get(0)
            .unwrap()
            .get("entity")
            .unwrap()
            .get("properties")
            .unwrap();

        let v = to_object(prop_map);
        return Ok(serde_json::from_value(v)?);
    };

    if let Some(missing) = v.get("missing") {
        let e: Entity =
            serde_json::from_value(missing.get(0).unwrap().get("entity").unwrap().clone())?;
        return Err(Error::new(
            StatusCode::NOT_FOUND,
            format!("result is missing: {}", e.to_string()),
        ));
    };

    // this must be: deferred
    Err(Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("could not deserialize lookup result, invalid result: {}", v).to_string(),
    ))
}

pub fn deserialize_query_result<D>(v: &Value) -> Result<D, Error>
where
    D: DeserializeOwned,
{
    if let Some(batch) = v.get("batch") {
        let results = batch.get("entityResults").unwrap().as_array().unwrap();
        let mut return_vec = Vec::<Value>::with_capacity(results.len());
        for r in results {
            let prop_map = r.get("entity").unwrap().get("properties").unwrap();
            return_vec.push(to_object(prop_map));
        }
        let return_arr = Value::Array(return_vec);
        return Ok(serde_json::from_value(return_arr)?);
    };

    Err(Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "could not deserialize qurey result, expect 'batch' and not: {:?}",
            v.get(0)
        ),
    ))
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
    use serde::{Deserialize, Serialize};

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

    #[derive(Deserialize, Serialize, Debug)]
    struct Hero {
        #[serde(rename(deserialize = "HeroID"))]
        hero_id: isize,
        #[serde(rename(deserialize = "Action"))]
        action: String,
        #[serde(rename(deserialize = "Time"))]
        time: String,
    }

    #[test]
    fn test_deserialize_lookup_result() {
        let json: &'static str = r#"{ "found": [ { "entity": {
        "properties": {
            "HeroID": { "integerValue": "0" },
            "Action": { "stringValue": "List" },
            "Time": { "timestampValue": "2018-07-27T20:13:20Z" }
        } } } ] }"#;

        let result_value: Value = serde_json::from_str(json).unwrap();
        let hero: Hero = deserialize_lookup_result(&result_value).unwrap();
        assert_eq!(0, hero.hero_id);
        assert_eq!("List", hero.action);
        assert_eq!("2018-07-27T20:13:20Z", hero.time);
    }

    #[test]
    fn test_deserialize_query_result() {
        let json: &'static str = r#"{ "batch": {
            "entityResultType": "FULL",
            "entityResults": [
              {
                "entity": {
                  "key": { "partitionId": { "projectId": "goheros-207118", "namespaceId": "heroes" },
                    "path": [ { "kind": "Protocol", "id": "5647341163905024" } ]
                  },
                  "properties": {
                    "Time": { "timestampValue": "2018-09-02T18:51:06Z" },
                    "HeroID": { "integerValue": "8"},
                    "Note": { "stringValue": "Delete Hero: &{8 {\n    \"name\": \"Foo-Bar\",\n} {  }} with ID: 8"},
                    "Action": {"stringValue": "Delete"}
                  }
                },
                "cursor": "CjgSMmoQZX5nb2hlcm9zLTIwNzExOHIVCxIIUHJvdG9jb2wYgICAoKGHhAoMogEGaGVyb2VzGAAgAA==", "version": "1535914685236000"
              },
              {
                "entity": {
                  "key": { "partitionId": { "projectId": "goheros-207118", "namespaceId": "heroes" },
                    "path": [ { "kind": "Protocol", "id": "5693417237512192" } ]
                  },
                  "properties": {
                    "Time": { "timestampValue": "2018-09-02T18:51:40Z" },
                    "HeroID": { "integerValue": "10" },
                    "Note": { "stringValue": "Delete Hero: &{10 {\n    \"New Foo-Bar2\"\n} {  }} with ID: 10" },
                    "Action": { "stringValue": "Delete" }
                  }
                },
                "cursor": "CjgSMmoQZX5nb2hlcm9zLTIwNzExOHIVCxIIUHJvdG9jb2wYgICAgKDEjgoMogEGaGVyb2VzGAAgAA==", "version": "1535914683774000" }
            ],
            "endCursor": "CjgSMmoQZX5nb2hlcm9zLTIwNzExOHIVCxIIUHJvdG9jb2wYgICAgKCSnwoMogEGaGVyb2VzGAAgAA==",
            "moreResults": "NO_MORE_RESULTS"
          } }"#;

        let result_value: Value = serde_json::from_str(json).unwrap();
        let heros: Vec<Hero> = deserialize_query_result(&result_value).unwrap();
        assert_eq!(2, heros.len());
        assert_eq!("2018-09-02T18:51:06Z", heros.get(0).unwrap().time);
        assert_eq!("Delete", heros.get(1).unwrap().action);
    }
}
