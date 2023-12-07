use validated::Validated::{self, Good, Fail};
use nonempty_collections::*;


// Here are a few design ideas to consider while implementing CQL generation.
// I am not sure how feasible/useful they are as I have a partial understanding
// of the spec and don't really know much about the constraints. Enola asked me to
// push it, so here it goes.

// Caveat: Did not touch JSON side, I see that as a separate issue. Also
// I was sloppy with the borrow checker, so probably there is room for memory
// footprint optimization.

// Some general remarks about safer Rust code.
// -------------------------------------------

// It is good practice to avoid using naked general purpose types like String.
// In the future we may want to restrict possible key values to, say, alphanumeric
// strings. Representing dates as naked strings is not kosher, either. The general
// idea is pushing preconditions upstream instead of implementing workarounds downstream.

// As an example, this is how I would define the Date type. Something similar can be done
// for id and key fields which are naked Strings.
mod safe_date {
    use chrono::NaiveDateTime;

    pub struct Date(String);

    impl Date {
        // Type comes with its validator but strictly speaking this is not necessary
        // as we do not process dates. If we start using optics in our Rust code
        // we can cast this mechanisms as a prism.
        fn new(str: String) -> Option<Date> {
            match NaiveDateTime::parse_from_str(&str, "%Y-%m-%d") {
                Ok(_) => Some(Date(str)),
                Err(_) => None,
            }
        }

        // An un-wrapper
        fn to_string(self) {
            self.0;
        }

        // Serialize/Deserialize would also be here. They can be implemented using new
        // and to_string above. Ideally we would also have a roundtrip test and some unit
        // tests. If we need to implement too many traits by hand we can use, for instance,
        // https://docs.rs/newtype_derive/latest/newtype_derive/
    }
}


// Now specific comments on the implementation.
// --------------------------------------------

// I changed the name to AstWithId because that's what it is.
pub struct AstWithId {
    pub ast: Ast,
    pub id: String, // Better be a 'newtype'
}

// Original AST definition was too complicated. An expression is
// either atomic or built from smaller expressions. No need for indirection.
pub enum Ast {
    Atomic(Condition),
    Compound(Operation, Vec<Ast>) // we can disallow empty vectors here but we have sane defaults so it is not a big deal
}

// Operand is the name of the inputs you give to
// to the operation in an expression. So changed this, too.
#[derive(Clone, Copy)]
pub enum Operation {
    And,
    Or,
}

// Having all the operation related things in one place is good.
// CQL support Xor. If we decide to implement it we only change here
// and the rest of the code works.
impl Operation {
    fn operation_name(self) -> &'static str {
        match self {
            Operation::And => " and ",
            Operation::Or => " or ",
        }
    }

    // this is not needed if we disallow empty vectors. some people find
    // this counterintuitive so maybe we should?
    fn nullary_value(self) -> &'static str {
        match self {
            Operation::And => "true", //empty iterator returns true under all in Rust
            Operation::Or => "false", //empty iterator returns false under any in Rust
        }
    }

    fn apply_to_group(self, group: Vec<&str>) -> String {
        if group.is_empty() {
            self.nullary_value().to_string()
        } else {
            group. // there should be a standard function for this somewhere
            iter().
            map(|s| s.to_string()).
            enumerate().
            map(|(index, s)| if index < group.len() - 1 {s + self.operation_name()} else {s}).
            collect::<Vec<String>>().
            concat()
        }
    }
}


// We can use some polymorphism here to avoid code duplication.
// and shine at cocktail parties.
pub struct AbstractRange<T> {
    pub min: T,
    pub max: T,
}

pub enum ConditionValue {
    DateRange(AbstractRange<safe_date::Date>),
    NumberRange(AbstractRange<f64>),
    Number(f64),
    //etc.
}

pub enum ConditionType {
    Equals,
    Between,
    //etc.
}

// We can have an enum of condition keys so we can reject unknown keys
// at json parsing level.
pub enum ConditionKey {
    Gender,
    Diagnosis,
    DiagnosisOld,
    // etc.
}

// Condition keys may depend on the project but we can always
// define `pup struct Condition<Key> {...}`..
pub struct Condition {
    key: ConditionKey,
    type_: ConditionType,
    value: ConditionValue
}

pub struct GeneratedCondition<'a> {
    retrieval: &'a str,
    filter: &'a str,
    code_systems: Vec<&'a str>, // This should probably be a set as we don't want duplicates.
}

// Specific errors about generation. At this level only incompatibility errors should be left.
// Everything else can be enforced by the type system so they can be pushed to the JSON parsing layer.
pub enum GenerationError {
    IncompatibleBlah,
    // etc.
}

// Generating texts from a condition is a standalone operation. Having
// a separated function for this makes hings cleaner.
pub fn generate_condition<'a>(condition: &Condition) -> Result<GeneratedCondition<'a>, GenerationError> {
    unimplemented!("All the table lookups, compatibility checks etc. should happen here");
}

// If we are fine with recursion we can use this. If we want a stack based implementation later
// it would be much easier to refactor. Error handling is streamlined and it is accumulative.
// As a middle solution we can bound the recursion depth, say by ~10.
// Since we will be reporting errors to a UI it is better to collect errors. This is what
// Validated does.
pub fn generate_all<'a>(ast: &Ast) -> Validated<GeneratedCondition<'a>, GenerationError> {
    match ast {
        Ast::Atomic(condition) =>
            match generate_condition(&condition){
                Ok(generated) => Good(generated),
                Err(err) => Fail(nev![err]),
            },
        Ast::Compound(op, vec_ast) =>
            {   let recursive_step: Validated<Vec<_>, _> = vec_ast.into_iter().map(generate_all).collect();
                match recursive_step {
                    Good(condition_vec) => {
                        let retrieval_vec: Vec<&str> = // i extracted the retrieval vec but you can generate all three needed vectors here in a single pass by a fold.
                            condition_vec.
                            into_iter().
                            map(|g| g.retrieval).
                            collect();
                        Good(GeneratedCondition
                            { retrieval: &format!("({})", op.apply_to_group(retrieval_vec))
                            , filter: todo!("Combine filters")
                            , code_systems: todo!("Combine code systems")
                            })},
                    Fail(errors) =>
                        Fail(errors),
            }},
    }
}
