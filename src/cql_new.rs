use crate::ast;
use crate::errors::FocusError;
use crate::projects::{Project, CriterionRole, CQL_TEMPLATES, MANDATORY_CODE_SYSTEMS, CODE_LISTS, CQL_SNIPPETS, CRITERION_CODE_LISTS, OBSERVATION_LOINC_CODE};

use chrono::offset::Utc;
use chrono::DateTime;
use indexmap::set::IndexSet;

pub struct GeneratedCondition<'a> {
    pub retrieval: Option<&'a str>, // Keep absence check for retrieval criteria at type level instead of inspecting the String later
    pub filter: Option<&'a str>, // Same as above
    pub code_systems: Vec<&'a str>, // This should probably be a set as we don't want duplicates.
  }

// Generating texts from a condition is a standalone operation. Having
// a separated function for this makes hings cleaner.
pub fn generate_condition<'a>(condition: &ast::Condition, project: Project) -> Result<GeneratedCondition<'a>, FocusError> {

    let mut code_systems: Vec<&str> = Vec::new();
    let mut filter: Option<&str> ;
    let mut retrieval: Option<&str>;


    //let generated_condition = GeneratedCondition::new();
    
    let condition_key_trans = condition.key.as_str();

    let condition_snippet =
        CQL_SNIPPETS.get(&(condition_key_trans, CriterionRole::Query, project.clone()));

    if let Some(snippet) = condition_snippet {
        let mut condition_string = (*snippet).to_string();
        let mut filter_string: String = "".to_string();

        let filter_snippet = CQL_SNIPPETS.get(&(
            condition_key_trans,
            CriterionRole::Filter,
            project.clone(),
        ));

        let code_lists_option = CRITERION_CODE_LISTS.get(&(condition_key_trans, project));
        if let Some(code_lists_vec) = code_lists_option {
            for (index, code_list) in code_lists_vec.iter().enumerate() {
                    code_systems.push(code_list);
                let placeholder =
                    "{{A".to_string() + (index + 1).to_string().as_str() + "}}"; //to keep compatibility with snippets in typescript
                condition_string =
                    condition_string.replace(placeholder.as_str(), code_list);
            }
        }

        if condition_string.contains("{{K}}") { 
            //observation loinc code, those only apply to query criteria, we don't filter specimens by observations
            let observation_code_option = OBSERVATION_LOINC_CODE.get(&condition_key_trans);

            if let Some(observation_code) = observation_code_option {
                condition_string = condition_string.replace("{{K}}", observation_code);
            } else {
                return Err(FocusError::AstUnknownOption(
                    condition_key_trans.to_string(),
                ));
            }
        }

        if let Some(filtret) = filter_snippet { 
            filter_string = (*filtret).to_string();
        }

        match condition.type_ {
            ast::ConditionType::Between => { // both min and max values stated
                match condition.value.clone() {
                    ast::ConditionValue::DateRange(date_range) => {
                        let datetime_str_min = date_range.min.as_str();
                        let datetime_result_min: Result<DateTime<Utc>, _> =
                            datetime_str_min.parse();

                        if let Ok(datetime_min) = datetime_result_min {
                            let date_str_min =
                                format!("@{}", datetime_min.format("%Y-%m-%d"));

                            condition_string =
                                condition_string.replace("{{D1}}", date_str_min.as_str());
                            filter_string =
                                filter_string.replace("{{D1}}", date_str_min.as_str()); // no condition needed, "" stays ""
                        } else {
                            return Err(FocusError::AstInvalidDateFormat(date_range.min));
                        }

                        let datetime_str_max = date_range.max.as_str();
                        let datetime_result_max: Result<DateTime<Utc>, _> =
                            datetime_str_max.parse();
                        if let Ok(datetime_max) = datetime_result_max {
                            let date_str_max =
                                format!("@{}", datetime_max.format("%Y-%m-%d"));

                            condition_string =
                                condition_string.replace("{{D2}}", date_str_max.as_str());
                            filter_string =
                                filter_string.replace("{{D2}}", date_str_max.as_str()); // no condition needed, "" stays ""
                        } else {
                            return Err(FocusError::AstInvalidDateFormat(date_range.max));
                        }
                    }
                    ast::ConditionValue::NumRange(num_range) => {
                        condition_string = condition_string
                            .replace("{{D1}}", num_range.min.to_string().as_str());
                        condition_string = condition_string
                            .replace("{{D2}}", num_range.max.to_string().as_str());
                        filter_string = filter_string
                            .replace("{{D1}}", num_range.min.to_string().as_str()); // no condition needed, "" stays ""
                        filter_string = filter_string
                            .replace("{{D2}}", num_range.max.to_string().as_str()); // no condition needed, "" stays ""
                    }
                    _ => {
                        return Err(FocusError::AstOperatorValueMismatch());
                    }
                }
            } // deal with no lower or no upper value
            ast::ConditionType::In => {
                // although in works in CQL, at least in some places, most of it is converted to multiple criteria with OR
                let operator_str = " or ";

                match condition.value.clone() {
                    ast::ConditionValue::StringArray(string_array) => {
                        let mut condition_humongous_string = "(".to_string();
                        let mut filter_humongous_string = "(".to_string();

                        for (index, string) in string_array.iter().enumerate() {
                            condition_humongous_string = condition_humongous_string
                                + "("
                                + condition_string.as_str()
                                + ")";
                            condition_humongous_string = condition_humongous_string
                                .replace("{{C}}", string.as_str());

                            filter_humongous_string = filter_humongous_string
                                + "("
                                + filter_string.as_str()
                                + ")";
                            filter_humongous_string =
                                filter_humongous_string.replace("{{C}}", string.as_str());

                            // Only concatenate operator if it's not the last element
                            if index < string_array.len() - 1 {
                                condition_humongous_string += operator_str;
                                filter_humongous_string += operator_str;
                            }
                        }
                        condition_string = condition_humongous_string + ")";

                        if !filter_string.is_empty() {
                            filter_string = filter_humongous_string + ")";
                        }
                    }
                    _ => {
                        return Err(FocusError::AstOperatorValueMismatch());
                    }
                }
            } // this becomes or of all
            ast::ConditionType::Equals => match condition.value.clone() {
                ast::ConditionValue::String(string) => {
                    condition_string = condition_string.replace("{{C}}", string.as_str());
                    filter_string = filter_string.replace("{{C}}", string.as_str()); // no condition needed, "" stays ""
                }
                _ => {
                    return Err(FocusError::AstOperatorValueMismatch());
                }
            },
            ast::ConditionType::NotEquals => { // won't get it from Lens
            }
            ast::ConditionType::Contains => { // won't get it from Lens
            }
            ast::ConditionType::GreaterThan => {} // guess Lens won't send me this
            ast::ConditionType::LowerThan => {}   // guess Lens won't send me this
        };

        retrieval = Some(condition_string.as_str());

        // if !filter.is_() && !filter_string.is_empty() {
        //     filter_cond += " and ";
        // }

        filter = Some(filter_string.as_str()); // no condition needed, "" can be added with no change
    } else {
        return Err(FocusError::AstUnknownCriterion(
            condition_key_trans.to_string(),
        ));
    }
    //if !filter_cond.is_empty() {
    //    filter_cond += " ";
    //}
    //retrieval_cond += " ";

    Ok( GeneratedCondition {
        retrieval: retrieval.clone(), // Keep absence check for retrieval criteria at type level instead of inspecting the String later
        filter: filter.clone(), // Same as above
        code_systems: code_systems.clone(),
    })
    

}


pub fn generate_cql(ast: ast::Ast, project: Project) -> Result<String, FocusError> {
    let mut retrieval_criteria: String = "".to_string(); // main selection criteria (Patient)

    let mut filter_criteria: String = "".to_string(); // criteria for filtering specimens

    let mut lists: String = "".to_string(); // needed code lists, defined

    let mut cql = CQL_TEMPLATES.get(&project).expect("missing project").to_string();

    let operator_str = match ast.ast.operand {
        ast::Operand::And => " and ",
        ast::Operand::Or => " or ",
    };

    let mut mandatory_codes = MANDATORY_CODE_SYSTEMS.get(&project)
        .expect("non-existent project")
        .clone();

    for (index, grandchild) in ast.ast.children.iter().enumerate() {
        process(
            grandchild.clone(),
            &mut retrieval_criteria,
            &mut filter_criteria,
            &mut mandatory_codes,
            project,
        )?;

        // Only concatenate operator if it's not the last element
        if index < ast.ast.children.len() - 1 {
            retrieval_criteria += operator_str;
        }
    }

    for code_system in mandatory_codes.iter() {
        lists += format!(
            "codesystem {}: '{}'\n",
            code_system,
            CODE_LISTS.get(code_system).unwrap_or(&(""))
        )
        .as_str();
    }

    cql = cql
        .replace("{{lists}}", lists.as_str());

    if retrieval_criteria.is_empty() {
        cql = cql.replace("{{retrieval_criteria}}", "true"); //()?
    } else {
        let formatted_retrieval_criteria = format!("({})", retrieval_criteria);
        cql = cql.replace("{{retrieval_criteria}}", formatted_retrieval_criteria.as_str());
    }


    if filter_criteria.is_empty() {
        cql = cql.replace("{{filter_criteria}}", "");
    } else {
        let formatted_filter_criteria = format!("where ({})", filter_criteria);
        dbg!(formatted_filter_criteria.clone());
        cql = cql.replace("{{filter_criteria}}", formatted_filter_criteria.as_str());
    }

    Ok(cql)
}

pub fn process(
    child: ast::Child,
    retrieval_criteria: &mut String,
    filter_criteria: &mut String,
    code_systems: &mut IndexSet<&str>,
    project: Project,
) -> Result<(), FocusError> {
    let mut retrieval_cond: String = "(".to_string();
    let mut filter_cond: String = "".to_string();

    match child {
        ast::Child::Condition(condition) => {
        }

        ast::Child::Operation(operation) => {
            let operator_str = match operation.operand {
                ast::Operand::And => " and ",
                ast::Operand::Or => " or ",
            };

            for (index, grandchild) in operation.children.iter().enumerate() {
                process(
                    grandchild.clone(),
                    &mut retrieval_cond,
                    &mut filter_cond,
                    code_systems,
                    project.clone(),
                )?;

                // Only concatenate operator if it's not the last element
                if index < operation.children.len() - 1 {
                    retrieval_cond += operator_str;
                    if !filter_cond.is_empty() {
                        filter_cond += operator_str;
                        dbg!(filter_cond.clone());
                    }
                }
            }
        }
    }

    retrieval_cond += ")";

    *retrieval_criteria += retrieval_cond.as_str();

    if !filter_cond.is_empty() { 
        dbg!(filter_cond.clone());
        *filter_criteria += "(";
        *filter_criteria += filter_cond.as_str();
        *filter_criteria += ")";

        dbg!(filter_criteria.clone());
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions;

    const AST: &str = r#"{"ast":{"operand":"AND","children":[{"key":"age","type":"EQUALS","value":5.0}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const MALE_OR_FEMALE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"},{"key":"gender","type":"EQUALS","system":"","value":"female"}]}]}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const ALL_GLIOMS: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"D43.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9383/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9384/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9394/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9421/1"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9382/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9391/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9400/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9424/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9425/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9450/3"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9440/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9441/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9442/3"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9381/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9382/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9401/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9451/3"}]}]}]}]}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const AGE_AT_DIAGNOSIS_30_TO_70: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"age_at_primary_diagnosis","type":"BETWEEN","system":"","value":{"min":30,"max":70}}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const AGE_AT_DIAGNOSIS_LOWER_THAN_70: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"age_at_primary_diagnosis","type":"BETWEEN","system":"","value":{"min":0,"max":70}}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const C61_OR_MALE: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C61"}]},{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const ALL_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["male","other"]},{"children":[{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C25"},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C56"}],"de":"Diagnose ICD-10","en":"Diagnosis ICD-10","key":"diagnosis","operand":"OR"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-09-30T22:00:00.000Z"}},{"key":"bmi","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"body_weight","system":"","type":"BETWEEN","value":{"max":1100,"min":10}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Other fasting status"]},{"key":"smoking_status","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":10000,"min":100}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","blood-plasma","buffy-coat"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-10-03T22:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature-18to-35","temperature-60to-85"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const SOME_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["other","male"]},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C24"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":11,"min":1}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"bmi","system":"","type":"BETWEEN","value":{"max":111,"min":1}},{"key":"body_weight","system":"","type":"BETWEEN","value":{"max":1111,"min":110}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Not sober"]},{"key":"smoking_status","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":123,"min":1}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","tissue-other"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature2to10","temperatureGN"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const LENS2: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"},{"key":"gender","system":"","type":"EQUALS","value":"female"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"tissue-frozen"},{"key":"sample_kind","system":"","type":"EQUALS","value":"blood-serum"}],"operand":"OR"}],"operand":"AND"},{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"liquid-other"},{"key":"sample_kind","system":"","type":"EQUALS","value":"rna"},{"key":"sample_kind","system":"","type":"EQUALS","value":"urine"}],"operand":"OR"},{"children":[{"key":"storage_temperature","system":"","type":"EQUALS","value":"temperatureRoom"},{"key":"storage_temperature","system":"","type":"EQUALS","value":"four_degrees"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const EMPTY: &str =
        r#"{"ast":{"children":[],"operand":"OR"}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    #[test]
    fn test_bbmri() {
        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(AST).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(MALE_OR_FEMALE).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(ALL_GLIOMS).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(AGE_AT_DIAGNOSIS_30_TO_70).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(AGE_AT_DIAGNOSIS_LOWER_THAN_70).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(C61_OR_MALE).expect("Failed to deserialize JSON"))
        // );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(ALL_GBN).expect("Failed to deserialize JSON"))
        // );

        // println!();

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(SOME_GBN).expect("Failed to deserialize JSON"))
        // );

        // println!();

        println!(
            "{:?}",
            generate_cql(serde_json::from_str(LENS2).expect("Failed to deserialize JSON"), Project::Bbmri)
        );

        // println!(
        //     "{:?}",
        //     bbmri(serde_json::from_str(EMPTY).expect("Failed to deserialize JSON"))
        // );

        pretty_assertions::assert_eq!(generate_cql(serde_json::from_str(EMPTY).unwrap(), Project::Bbmri).unwrap(), include_str!("../resources/test/result_empty.cql").to_string());

    }
}
