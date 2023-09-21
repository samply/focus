use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::errors::FocusError;


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde()]
enum Child {
    Operation(Operation),
    Condition(Condition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
enum Operand {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
#[serde()]
enum ConditionValue {
    String(String),
    StringArray(Vec<String>),
    Boolean(bool),
    Number(f64),
    NumRange(NumRange),
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
    children: Vec<Child>,
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

    const EQUALS_AST: &str = r#"{"ast":{"operand":"AND","children":[{"Condition":{"key":"age","type_":"EQUALS","value":{"Number":5.0}}}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;


    #[test]
    fn test_deserialize_ast() {
        let ast_variable: Ast = serde_json::from_str(EQUALS_AST).expect("Failed to deserialize JSON");

        let ast_string = serde_json::to_string(&ast_variable).expect("Failed to serialize JSON");

        assert_eq!(EQUALS_AST, ast_string);
    }


}
