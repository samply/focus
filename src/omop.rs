use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::FocusError;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Child {
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
#[serde(untagged)]
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
#[serde(rename_all = "camelCase")]
pub struct Condition {
    key: String,
    type_: ConditionType,
    value: ConditionValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ast {
    ast: Operation,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OmopQuery{
    pub lang: String,
    pub query: String
}



pub async fn post_ast(ast: Ast) -> Result<String, FocusError> {
    debug!("Posting AST...");

    let ast_string = serde_json::to_string_pretty(&ast)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;

    debug!("{}", ast_string.clone());

    let resp = CONFIG
        .client
        .post(format!("{}", CONFIG.endpoint_url))
        .header("Content-Type", "application/json")
        .body(ast_string.clone())
        .send()
        .await
        .map_err(|e| FocusError::UnableToPostAst(e))?;

    debug!("Posted AST...");

    dbg!(resp.status().clone());

    let text = match resp.status() {
        StatusCode::OK => {
            resp
            .text()
            .await
            .map_err(|e| FocusError::UnableToPostAst(e))?
        },
        code => {
            warn!("Got unexpected code {code} while posting AST; reply was `{}`, debug info: {:?}", ast_string, resp);
            return Err(FocusError::AstPostingErrorReqwest(format!(
                "Error while posting AST `{}`: {:?}",
                ast_string, resp
            )));
        }
    };

    dbg!(text.clone());

    Ok(text)
}

#[cfg(test)]
mod test {
    use super::*;

    const EQUALS_AST: &str = r#"{"ast":{"operand":"AND","children":[{"key":"age","type":"EQUALS","value":5.0}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;
    const LENS_AST: &str = r#"[{"key":"gender","system":"","type":"IN","value":["M","SCTID:261665006"]},{"key":"age_group","system":"","type":"BETWEEN","value":{"max":70,"min":40}},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C61"},{"key":"modality","system":"urn:oid:2.16.840.1.113883.6.256","type":"IN","value":["RID10312"]}]"#;
    const LENS_AST_TRANS: &str = r#"{"ast":{"operand":"AND","children":[{"key":"gender","type":"IN","value":["M","SCTID:261665006"]},{"key":"age_group","type":"BETWEEN","value":{"min":40.0,"max":70.0}},{"key":"diagnosis","type":"EQUALS","value":"C61"},{"key":"modality","type":"IN","value":["RID10312"]}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;


    #[test]
    fn test_deserialize_ast() {
        let ast_variable: Ast =
            serde_json::from_str(EQUALS_AST).expect("Failed to deserialize JSON");

        let ast_string = serde_json::to_string(&ast_variable).expect("Failed to serialize JSON");

        assert_eq!(EQUALS_AST, ast_string);
    }

    fn replace_uuid_test(trans_ast: String) -> Result<String, FocusError>{

        let mut uuid_ast = trans_ast.replace("uuid", "a6f1ccf3-ebf1-424f-9d69-4e5d135f2340");
    
        Ok(uuid_ast)
    
    }


}
