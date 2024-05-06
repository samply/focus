use serde::Deserialize;
use serde::Serialize;


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Child {
    Operation(Operation),
    Condition(Condition),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum Operand { //this is operator, of course, but rename would need to be coordinated with all the Lenses, EUCAIM providers, etc
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConditionType {
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
pub enum ConditionValue {
    String(String),
    StringArray(Vec<String>),
    Boolean(bool),
    Number(f64),
    NumRange(NumRange),
    DateRange(DateRange),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DateRange {
    pub min: String, // we don't parse dates yet
    pub max: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operation {
    pub operand: Operand,
    pub children: Vec<Child>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub key: String,
    pub type_: ConditionType,
    pub value: ConditionValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ast {
    pub ast: Operation,
    pub id: String,
}



#[cfg(test)]
mod test {
    use super::*;

    const EQUALS_AST: &str = r#"{"ast":{"operand":"AND","children":[{"key":"age","type":"EQUALS","value":5.0}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;


    #[test]
    fn test_deserialize_ast() {
        let ast_variable: Ast =
            serde_json::from_str(EQUALS_AST).expect("Failed to deserialize JSON");

        let ast_string = serde_json::to_string(&ast_variable).expect("Failed to serialize JSON");

        assert_eq!(EQUALS_AST, ast_string);
    }


}
