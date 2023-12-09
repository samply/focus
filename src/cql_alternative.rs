use validated::Validated::{self, Good, Fail};
use nonempty_collections::*;
use focus_api::*;

pub struct GeneratedCondition<'a> {
  pub retrieval: Option<&'a str>, // Keep absence check for retrieval criteria at type level instead of inspecting the String later
  pub filter: Option<&'a str>, // Same as above
  pub code_systems: Vec<&'a str>, // This should probably be a set as we don't want duplicates.
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
                            flatten().
                            collect();
                        Good(GeneratedCondition
                            { retrieval:
                                  if retrieval_vec.is_empty()
                                      { None } else
                                      { Some (&format!("({})", op.apply_to_group(retrieval_vec))) }
                            , filter: todo!("Combine filters")
                            , code_systems: todo!("Combine code systems")
                            })},
                    Fail(errors) =>
                        Fail(errors),
            }},
    }
}
