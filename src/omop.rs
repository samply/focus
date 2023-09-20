use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::errors::FocusError;
use crate::util::get_json_field;
use crate::config::CONFIG;

enum ChildType {
    Operation(Operation),
    Condition(Condition),
}

enum Operand {
    And,
    Or,
}

enum ConditionType {
    Equals,
    NotEquals,
    In,
    Between,
    LowerThan,
    GreaterThan,
    Contains,
}

enum ConditionValue{
    String(String),
    StringArray(Vec<String>),
    Boolean(Boolean),
    Number(f64),
    NumRange(NumRange),
    Date(String),
    DateRange(DateRange)
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NumRange {
    min: f64, 
    max: f64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DateRange {
    min: String, //we don't parse dates yet
    max: String,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Operation {
    operand: Operand, 
    children: Vec<ChildType>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Condition {
    key: String,
    type_: ConditionType,
    value: ConditionValue,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Ast {
    ast: Operation,
    id: Uuid
}



pub async fn post_ast(ast: Ast) -> Result<(), FocusError> {
    debug!("Posting AST...");

    Ok(())
}



