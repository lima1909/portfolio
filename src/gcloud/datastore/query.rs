use super::converter::deserialize_query_result;
use super::{Error, ReadConsistency, ResponseError};
use http::StatusCode;
use reqwest::blocking;
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

const QUERY_JSON: &'static str = r#"{
    "partitionId": { "namespaceId": "{namespace}" },
    "readOptions": { "readConsistency": "{readConsistency}" },
    "query": { "kind": { "name": "{kind}"},
    {filter}
    }
}"#;

#[allow(dead_code)]
pub enum Operator {
  OperatorUnspecified,
  LessThan,
  LessThanOrEqual,
  GreaterThan,
  GreaterThanOrEqual,
  Equal,
  HasAncestor,
}

impl Operator {
  #[allow(dead_code)]
  fn to_json(&self) -> String {
    match self {
      Operator::OperatorUnspecified => r#""op":"OPERATOR_UNSPECIFIED""#.to_string(),
      Operator::LessThan => r#""op":"LESS_THAN""#.to_string(),
      Operator::LessThanOrEqual => r#""op":"LESS_THAN_OR_EQUAL""#.to_string(),
      Operator::GreaterThan => r#""op":"GREATER_THAN""#.to_string(),
      Operator::GreaterThanOrEqual => r#""op":"GREATER_THAN_OR_EQUAL""#.to_string(),
      Operator::Equal => r#""op":"EQUAL""#.to_string(),
      Operator::HasAncestor => r#""op":"HAS_ANCESTOR""#.to_string(),
    }
  }
}

#[allow(dead_code)]
pub enum Value {
  Null,
  Bool(bool),
  String(String),
  Integer(isize),
  Double(f64),
}

impl Value {
  #[allow(dead_code)]
  fn to_json(&self) -> String {
    match self {
      Value::Null => r#""value":{"nullValue":null}"#.to_string(),
      Value::Bool(v) => r#""value":{"booleanValue":"{value}"}"#.replace("{value}", &v.to_string()),
      Value::String(v) => r#""value":{"stringValue":"{value}"}"#.replace("{value}", &v),
      Value::Integer(v) => r#""value":{"integerValue":{value}}"#.replace("{value}", &v.to_string()),
      Value::Double(v) => r#""value":{"doubleValue":{value}}"#.replace("{value}", &v.to_string()),
    }
  }
}

pub struct Filter<'a> {
  pub property: &'a str,
  pub op: Operator,
  pub value: Value,
}

#[allow(dead_code)]
impl<'a> Filter<'a> {
  pub fn to_json(&self) -> String {
    format!(
      "{} {}, {} {}",
      r#""filter":{"propertyFilter":{"property":{"name":"{property}"},"#
        .replace("{property}", self.property),
      self.op.to_json(),
      self.value.to_json(),
      "} }",
    )
  }
}

fn create_query_json(namespace: &str, kind: &str, filter: &str) -> String {
  QUERY_JSON
    .replace("{readConsistency}", ReadConsistency::Eventual.to_string())
    .replace("{namespace}", namespace)
    .replace("{kind}", kind)
    .replace("{filter}", filter)
    .replace("\n", "")
}

pub fn query<D: DeserializeOwned>(
  client: &blocking::Client,
  auth_query_str: &str,
  project: &str,
  namespace: &str,
  kind: &str,
  filter: &Filter,
) -> Result<Vec<D>, Error> {
  let url = format!(
    "https://datastore.googleapis.com/v1/projects/{}:runQuery?{}",
    project, auth_query_str
  );
  let lookup_json = create_query_json(namespace, kind, &filter.to_json());
  let resp = client.post(&url).body(lookup_json).send()?;

  if resp.status().as_u16() == StatusCode::OK.as_u16() {
    let v = resp.json::<JsonValue>().unwrap();
    return Ok(deserialize_query_result(&v)?);
  } else {
    Err(resp.json::<ResponseError>()?.error)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::Value as JsonValue;

  #[test]
  fn test_filter() {
    let f = Filter {
      property: "Action",
      op: Operator::Equal,
      value: Value::String(String::from("List")),
    };
    let json_str = format!("{} {} {}", "{", f.to_json(), "}");
    let r: JsonValue = serde_json::from_str(&json_str).unwrap();
    r.get("Filter");
  }

  #[test]
  fn test_operator() {
    assert_eq!(
      r#""op":"OPERATOR_UNSPECIFIED""#.to_string(),
      Operator::OperatorUnspecified.to_json()
    );
    assert_eq!(
      r#""op":"LESS_THAN""#.to_string(),
      Operator::LessThan.to_json()
    );
    assert_eq!(
      r#""op":"LESS_THAN_OR_EQUAL""#.to_string(),
      Operator::LessThanOrEqual.to_json()
    );
    assert_eq!(
      r#""op":"GREATER_THAN""#.to_string(),
      Operator::GreaterThan.to_json()
    );
    assert_eq!(
      r#""op":"GREATER_THAN_OR_EQUAL""#.to_string(),
      Operator::GreaterThanOrEqual.to_json()
    );
    assert_eq!(r#""op":"EQUAL""#.to_string(), Operator::Equal.to_json());
    assert_eq!(
      r#""op":"HAS_ANCESTOR""#.to_string(),
      Operator::HasAncestor.to_json()
    );
  }

  #[test]
  fn test_value() {
    assert_eq!(
      r#""value":{"nullValue":null}"#.to_string(),
      Value::Null.to_json()
    );
    assert_eq!(
      r#""value":{"booleanValue":"true"}"#.to_string(),
      Value::Bool(true).to_json()
    );
    assert_eq!(
      r#""value":{"stringValue":"Foo"}"#.to_string(),
      Value::String(String::from("Foo")).to_json()
    );
    assert_eq!(
      r#""value":{"integerValue":42}"#.to_string(),
      Value::Integer(42).to_json()
    );
    assert_eq!(
      r#""value":{"doubleValue":4.2}"#.to_string(),
      Value::Double(4.2).to_json()
    );
  }
}
