// Mirrors the AST in lens3 (packages/lens/src/types/ast.ts).

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub expression: Expression,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Expression {
    And {
        operands: Vec<Expression>,
    },
    Or {
        operands: Vec<Expression>,
    },
    Not {
        operand: Box<Expression>,
    },
    MatchAll,
    Exists(Exists),
    Subject {
        operand: Predicate,
    },
    /// Operands must be `exists`; that's checked when generating CQL.
    Temporal {
        first: Box<Expression>,
        steps: Vec<Step>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Exists {
    pub scope: String,
    pub operand: Option<Predicate>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "operator", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Then { operand: Expression },
    WithinDays { days: u32, operand: Expression },
    AtLeastDays { days: u32, operand: Expression },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Predicate {
    And {
        operands: Vec<Predicate>,
    },
    Or {
        operands: Vec<Predicate>,
    },
    Values {
        field: String,
        values: Vec<String>,
    },
    Date {
        field: String,
        min: Option<String>,
        max: Option<String>,
    },
    Range {
        field: String,
        min: Option<f64>,
        max: Option<f64>,
    },
}
