//! # [Presentation Exchange] Constraints
//!
//! Checks whether a `VerifiableCredential` satisfies constraints specified in a
//! [Presentation Definition].
//!
//! [Presentation Exchange]: (https://identity.foundation/presentation-exchange/spec/v2.0.0)
//! [Presentation Definition]: (https://identity.foundation/presentation-exchange/spec/v2.0.0/#presentation-definition)

use anyhow::{Result, anyhow};
use chrono::{DateTime, NaiveDate};
use regex::Regex;
use serde_json::Value;
use serde_json_path::JsonPath;

use super::{Claims, Constraints, Field, FilterValue};

// LATER: add support for Zero-Knowledge Proofs by enabling the `predicate`
// feature

impl Constraints {
    /// Check if a `VerifiableCredential` satisfies constraints provided in the
    /// Presentation Definition.
    ///
    /// # Errors
    ///
    /// Returns an error if the `JSONPath` query is invalid.
    pub fn satisfied(&self, claims: &impl Claims) -> Result<bool> {
        let Some(fields) = &self.fields else {
            return Ok(true);
        };
        let Ok(vc_val) = claims.to_json() else {
            return Err(anyhow!("error serializing credential"));
        };

        // EVERY field must match
        for field in fields {
            // if no match AND the field is not optional, constraints are not satisfied
            if !field.matched(&vc_val).unwrap_or_default() && !field.optional.unwrap_or_default() {
                return Ok(false);
            }
        }

        // all fields match
        Ok(true)
    }
}

impl Field {
    /// Check whether `Constraint` `Field` can be matched to a field the
    /// provided `VerifiableCredential`.
    /// The [Presentation Exchange 2.0.0] specification only requires one
    /// matching JSON path expression for the field to be considered
    /// matched.
    fn matched(&self, vc: &Value) -> Result<bool> {
        // find the FIRST matching JSON path expression for field
        for path in &self.path {
            // execute JSONPath query
            let Ok(jpath) = JsonPath::parse(path) else {
                return Err(anyhow!("Invalid JSONPath: {path}"));
            };
            let nodes = jpath.query(vc).all();

            // no matches: try next path
            if nodes.is_empty() {
                continue;
            }

            // no filter == match(?)
            let Some(filter) = &self.filter else {
                return Ok(true);
            };

            // find FIRST node matching filter (in practice, there should only be one node)
            if let Some(node) = nodes.into_iter().next() {
                match filter.value.matched(node) {
                    Ok(true) => return Ok(true),
                    Ok(false) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(false)
    }
}

impl FilterValue {
    /// Check whether the result of a `JSONPath` query can be matched to the
    /// specified `FilterValue`.
    fn matched(&self, vc_node: &Value) -> Result<bool> {
        match self {
            Self::Const(_) => match_const(self, vc_node),
            Self::Pattern(_) => match_pattern(self, vc_node),
            Self::Format(_) => match_format(self, vc_node),
        }
    }
}

// TODO: check filter.type

// Check whether JSONPath result matches (exactly) `FilterValue`.
fn match_const(filter_val: &FilterValue, vc_node: &Value) -> Result<bool> {
    let FilterValue::Const(const_str) = filter_val else {
        return Err(anyhow!("unexpected filter"));
    };
    let Ok(const_val) = serde_json::to_value(const_str) else {
        return Err(anyhow!("issue serializing match criteria"));
    };

    match vc_node {
        Value::Array(arr_node) => Ok(arr_node.contains(&const_val)),
        Value::String(str_node) => Ok(str_node == &const_val.to_string()),
        Value::Number(num_node) => Ok(num_node.to_string() == const_val),
        Value::Bool(bool_node) => Ok(Some(*bool_node) == const_val.as_bool()),
        Value::Null => Ok(true),
        Value::Object(_) => {
            unimplemented!("object matching not implemented")
        }
    }
}

// Check whether JSONPath result matches regex specified by `FilterValue`.
fn match_pattern(filter_val: &FilterValue, vc_node: &Value) -> Result<bool> {
    let FilterValue::Pattern(pattern) = filter_val else {
        return Err(anyhow!("unexpected filter"));
    };
    let Ok(re) = Regex::new(pattern) else {
        return Err(anyhow!("invalid regex pattern: {pattern}"));
    };

    Ok(re.captures(&vc_node.to_string()).is_some())
}

// Check whether JSONPath result format matches that specified by `FilterValue`.
fn match_format(filter_val: &FilterValue, vc_node: &Value) -> Result<bool> {
    let FilterValue::Format(format) = filter_val else {
        return Err(anyhow!("unexpected filter"));
    };

    match format.as_str() {
        "date" => match vc_node {
            Value::String(str_node) => Ok(NaiveDate::parse_from_str(str_node, "%Y-%m-%d").is_ok()),
            _ => Ok(false),
        },
        "date-time" => match vc_node {
            Value::String(str_node) => Ok(DateTime::parse_from_rfc3339(str_node).is_ok()),
            _ => Ok(false),
        },
        _ => unimplemented!("format matching not implemented for {format}"),
    }
}

#[cfg(test)]
mod test {

    use serde_json::json;

    use super::*;

    #[test]
    fn test_const() {
        let constr = json!({
            "fields": [{
                "path":["$.type"],
                "filter": {
                    "type": "string",
                    "const": "EmployeeIDCredential"
                }
            }]
        });

        let constraints: Constraints = serde_json::from_value(constr).expect("should deserialize");
        assert!(constraints.satisfied(&Credential).unwrap());
    }

    #[test]
    fn test_pattern() {
        let constr = json!({
            "fields": [{
                "path":["$.type"],
                "filter": {
                    "type": "string",
                    "pattern": "EmployeeID[a-zA-Z]+"
                }
            }]
        });

        let constraints: Constraints = serde_json::from_value(constr).expect("should deserialize");
        assert!(constraints.satisfied(&Credential).unwrap());
    }

    #[test]
    fn test_format() {
        let constr = json!({
            "fields": [{
                "path":["$.validFrom"],
                "filter": {
                    "type": "string",
                    "format": "date-time"
                }
            }]
        });

        let constraints: Constraints = serde_json::from_value(constr).expect("should deserialize");

        assert!(constraints.satisfied(&Credential).unwrap());
    }

    struct Credential;
    impl Claims for Credential {
        fn to_json(&self) -> Result<Value> {
            Ok(json!({
                "@context":[
                    "https://www.w3.org/2018/credentials/v1",
                    "https://www.w3.org/2018/credentials/examples/v1"
                ],
                "type":[
                    "VerifiableCredential",
                    "EmployeeIDCredential"
                ],
                "id":"https://example.com/credentials/3732",
                "issuer":"https://example.com/issuers/14",
                "validFrom":"2023-11-20T23:21:55Z",
                "validUntil":"2023-12-20T23:21:55Z",
                "credentialSubject":{
                    "employeeId":"1234567890",
                    "id":"did:example:ebfeb1f712ebc6f1c276e12ec21"
                }
            }))
        }
    }
}
