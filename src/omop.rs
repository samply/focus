use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::FocusError;
use crate::util::get_json_field;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ChildType {
    Operation(Operation),
    Condition(Condition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Operand {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ConditionType {
    Equals,
    NotEquals,
    In,
    Between,
    LowerThan,
    GreaterThan,
    Contains,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum ConditionValue {
    String(String),
    StringArray(Vec<String>),
    Boolean(bool),
    Number(f64),
    NumRange(NumRange),
    Date(String),
    DateRange(DateRange),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumRange {
    min: f64,
    max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateRange {
    min: String, //we don't parse dates yet
    max: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    operand: Operand,
    children: Vec<ChildType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
    key: String,
    type_: ConditionType,
    value: ConditionValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ast {
    ast: Operation,
    id: Uuid,
}

pub async fn post_ast(ast: Ast) -> Result<(), FocusError> {
    debug!("Posting AST...");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    const EXAMPLE_AST: &str = r#"{
        "ast": {
            "operand": "And",
            "children": [
                {
                    "type": "Operation",
                    "operand": "Or",
                    "children": [
                        {
                            "type": "Condition",
                            "key": "gender",
                            "type": "Equals",
                            "value": {
                                "type": "String",
                                "String": "male"
                            }
                        },
                        {
                            "type": "Condition",
                            "key": "age",
                            "type": "GreaterThan",
                            "value": {
                                "type": "Number",
                                "Number": 42
                            }
                        }
                    ]
                },
                {
                    "type": "Condition",
                    "key": "diagnosis",
                    "type": "In",
                    "value": {
                        "type": "StringArray",
                        "StringArray": ["C61", "C34.0"]
                    }
                }
            ]
        },
        "id": "a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"
    }
    "#;

    #[test]
    fn test_deserialize_ast() {
        let ast_variable: Ast = serde_json::from_str(EXAMPLE_AST).expect("Failed to deserialize JSON");

        let ast_string = serde_json::to_string(&ast_variable).expect("Failed to serialize JSON");
        //.expect("Failed to serialize to JSON");

        assert_eq!(EXAMPLE_AST, ast_string);
    }
}
