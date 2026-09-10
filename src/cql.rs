use crate::ast;
use crate::errors::FocusError;
use crate::flavours::{CriterionRole, Flavour};

use base64::{prelude::BASE64_STANDARD as BASE64, Engine as _};
use chrono::offset::Utc;
use chrono::{DateTime, NaiveDate, NaiveTime};
use indexmap::set::IndexSet;
use tracing::info;
use uuid::Uuid;

pub fn generate_body(ast: ast::Ast, project: Flavour) -> Result<String, FocusError> {
    Ok(project
        .get_body()
        .replace(
            "{{LIBRARY_UUID}}",
            format!("urn:uuid:{}", Uuid::new_v4()).as_str(),
        )
        .replace(
            "{{MEASURE_UUID}}",
            format!("urn:uuid:{}", Uuid::new_v4()).as_str(),
        )
        .replace(
            "{{LIBRARY_ENCODED}}",
            BASE64.encode(generate_cql(ast, project)?).as_str(),
        ))
}

fn generate_cql(ast: ast::Ast, cql_flavour: Flavour) -> Result<String, FocusError> {
    let mut retrieval_criteria: String = String::new(); // main selection criteria (Patient)

    let mut filter_criteria: String = String::new(); // criteria for filtering specimens

    let mut lists: String = String::new(); // needed code lists, defined

    let mut cql = cql_flavour.get_cql_template().to_string();

    let operator_str = match ast.ast.operand {
        ast::Operand::And => " and ",
        ast::Operand::Or => " or ",
    };

    let mut mandatory_codes = cql_flavour.get_mandatory_code_lists().clone();

    for (index, grandchild) in ast.ast.children.iter().enumerate() {
        process(
            grandchild.clone(),
            &mut retrieval_criteria,
            &mut filter_criteria,
            &mut mandatory_codes,
            &cql_flavour,
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
            cql_flavour
                .get_code_lists()
                .get(code_system)
                .unwrap_or(&(""))
        )
        .as_str();
    }

    cql = cql.replace("{{lists}}", lists.as_str());

    if retrieval_criteria.is_empty()
        || retrieval_criteria
            .replace("or", " ")
            .chars()
            .all(|c| [' ', '(', ')'].contains(&c))
    {
        //to deal with an empty criteria tree of an arbitrary depth
        cql = cql.replace("{{retrieval_criteria}}", "true"); //()?
    } else {
        let formatted_retrieval_criteria = format!("({})", retrieval_criteria);
        cql = cql.replace(
            "{{retrieval_criteria}}",
            formatted_retrieval_criteria.as_str(),
        );
    }

    if filter_criteria.is_empty() {
        cql = cql.replace("{{filter_criteria}}", "");
    } else {
        let formatted_filter_criteria = format!("where ({})", filter_criteria);
        cql = cql.replace("{{filter_criteria}}", formatted_filter_criteria.as_str());
    }

    Ok(cql)
}

pub fn process(
    child: ast::Child,
    retrieval_criteria: &mut String,
    filter_criteria: &mut String,
    code_systems: &mut IndexSet<&str>,
    cql_flavour: &Flavour,
) -> Result<(), FocusError> {
    let mut retrieval_cond: String = "(".to_string();
    let mut filter_cond: String = String::new();

    match child {
        ast::Child::Condition(condition) => {
            let condition_key_trans = condition.key.as_str();

            let condition_snippet = cql_flavour
                .get_cql_snippets()
                .get(&(condition_key_trans, CriterionRole::Query));

            let Some(snippet) = condition_snippet else {
                return Err(FocusError::AstUnknownCriterion(
                    condition_key_trans.to_string(),
                ));
            };
            let mut condition_string = (*snippet).to_string();
            let mut filter_string: String = String::new();

            let filter_snippet = cql_flavour
                .get_cql_snippets()
                .get(&(condition_key_trans, CriterionRole::Filter));

            let code_lists_option = cql_flavour
                .get_criterion_code_lists()
                .get(&(condition_key_trans));
            if let Some(code_lists_vec) = code_lists_option {
                for (index, code_list) in code_lists_vec.iter().enumerate() {
                    code_systems.insert(code_list);
                    let placeholder = format!("{{{{A{}}}}}", (index + 1)); //to keep compatibility with snippets in typescript
                    condition_string = condition_string.replace(placeholder.as_str(), code_list);
                }
            }

            if condition_string.contains("{{K}}") {
                //observation loinc code, those only apply to query criteria, we don't filter specimens by observations
                let observation_code_option = cql_flavour
                    .get_observation_loinc_codes()
                    .get(&condition_key_trans);

                if let Some(observation_code) = observation_code_option {
                    condition_string = condition_string.replace("{{K}}", &escape(observation_code));
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
                ast::ConditionType::Between => {
                    // both min and max values stated
                    match condition.value {
                        ast::ConditionValue::DateRange(date_range) => {
                            // check which values exist
                            let wrapped_min = date_range.min;
                            let wrapped_max = date_range.max;

                            match (wrapped_min, wrapped_max) {
                                (None, None) => {
                                    return Err(FocusError::NoMinNoMax);
                                }
                                (None, Some(max)) => {
                                    condition_string = condition_string
                                        .replace("between {{D1}} and {{D2}}", " <= {{D2}}");
                                    condition_string = condition_string.replace(
                                        "{{D2}}",
                                        formated_date_string(max.as_str())?.as_str(),
                                    ); // no CQL injection possible here

                                    filter_string = filter_string
                                        .replace("between {{D1}} and {{D2}}", " <= {{D2}}"); // no condition needed, "" stays ""
                                    filter_string = filter_string.replace(
                                        "{{D2}}",
                                        formated_date_string(max.as_str())?.as_str(),
                                    );
                                    // no condition needed, "" stays ""; no CQL injection possible here
                                }
                                (Some(min), None) => {
                                    condition_string = condition_string
                                        .replace("between {{D1}} and {{D2}}", " >= {{D1}}");
                                    condition_string = condition_string.replace(
                                        "{{D1}}",
                                        formated_date_string(min.as_str())?.as_str(),
                                    ); // no CQL injection possible here

                                    filter_string = filter_string
                                        .replace("between {{D1}} and {{D2}}", " >= {{D1}}"); // no condition needed, "" stays ""
                                    filter_string = filter_string.replace(
                                        "{{D1}}",
                                        formated_date_string(min.as_str())?.as_str(),
                                    );
                                    // no condition needed, "" stays ""; no CQL injection possible here
                                }
                                (Some(min), Some(max)) => {
                                    condition_string = condition_string.replace(
                                        "{{D1}}",
                                        formated_date_string(min.as_str())?.as_str(),
                                    ); // no CQL injection possible here
                                    condition_string = condition_string.replace(
                                        "{{D2}}",
                                        formated_date_string(max.as_str())?.as_str(),
                                    ); // no CQL injection possible here
                                    filter_string = filter_string.replace(
                                        "{{D1}}",
                                        formated_date_string(min.as_str())?.as_str(),
                                    ); // no condition needed, "" stays ""; no CQL injection possible here
                                    filter_string = filter_string.replace(
                                        "{{D2}}",
                                        formated_date_string(max.as_str())?.as_str(),
                                    );
                                    // no CQL injection possible here; no condition needed, "" stays ""
                                }
                            }
                        }
                        ast::ConditionValue::NumRange(num_range) => {
                            // check which values exist
                            let wrapped_min = num_range.min;
                            let wrapped_max = num_range.max;

                            match (wrapped_min, wrapped_max) {
                                (None, None) => {
                                    return Err(FocusError::NoMinNoMax);
                                }
                                (None, Some(max)) => {
                                    let max = max.to_string();
                                    condition_string = condition_string
                                        .replace("between {{D1}} and {{D2}}", " <= {{D2}}")
                                        .replace(
                                            "between Ceiling({{D1}}) and Ceiling({{D2}}",
                                            " <= Ceiling({{D2}}",
                                        );
                                    condition_string =
                                        condition_string.replace("{{D2}}", max.as_str()); // no CQL injection possible here

                                    filter_string = filter_string
                                        .replace("between {{D1}} and {{D2}}", " <= {{D2}}")
                                        .replace(
                                            "between Ceiling({{D1}}) and Ceiling({{D2}}",
                                            " <= Ceiling({{D2}}",
                                        ); // no condition needed, "" stays ""
                                    filter_string = filter_string.replace("{{D2}}", max.as_str());
                                    // no condition needed, "" stays ""; no CQL injection possible here
                                }
                                (Some(min), None) => {
                                    let min = min.to_string();
                                    condition_string = condition_string
                                        .replace("between {{D1}} and {{D2}}", " >= {{D1}}")
                                        .replace(
                                            "between Ceiling({{D1}}) and Ceiling({{D2}}",
                                            " >= Ceiling({{D1}}",
                                        );
                                    condition_string =
                                        condition_string.replace("{{D1}}", min.as_str()); // no CQL injection possible here

                                    filter_string = filter_string
                                        .replace("between {{D1}} and {{D2}}", " >= {{D1}}")
                                        .replace(
                                            "between Ceiling({{D1}}) and Ceiling({{D2}}",
                                            " >= Ceiling({{D1}}",
                                        ); // no condition needed, "" stays ""
                                    filter_string = filter_string.replace("{{D1}}", min.as_str());
                                    // no condition needed, "" stays ""; no CQL injection possible here
                                }
                                (Some(min), Some(max)) => {
                                    let min = min.to_string();
                                    let max = max.to_string();

                                    condition_string =
                                        condition_string.replace("{{D1}}", min.as_str()); // no CQL injection possible here
                                    condition_string =
                                        condition_string.replace("{{D2}}", max.as_str()); // no CQL injection possible here
                                    filter_string = filter_string.replace("{{D1}}", min.as_str()); // no condition needed, "" stays ""; no CQL injection possible here
                                    filter_string = filter_string.replace("{{D2}}", max.as_str());
                                    // no CQL injection possible here; no condition needed, "" stays ""
                                }
                            }
                        }
                        other => {
                            return Err(FocusError::AstOperatorValueMismatch(format!("Operator BETWEEN can only be used for numerical and date values, not for {:?}", other)));
                        }
                    }
                } // deal with no lower or no upper value
                ast::ConditionType::In => {
                    // although in works in CQL, at least in some places, most of it is converted to multiple criteria with OR
                    let operator_str = " or ";

                    match condition.value {
                        ast::ConditionValue::StringArray(string_array) => {
                            let mut string_array_with_workarounds =
                                if *cql_flavour == Flavour::Miabis {
                                    //empty, codes get replaced
                                    Default::default()
                                } else {
                                    string_array.clone()
                                };

                            for value in string_array {
                                if let Some(additional_values) =
                                    cql_flavour.get_code_workarounds().get(value.as_str())
                                {
                                    for additional_value in additional_values {
                                        string_array_with_workarounds
                                            .push((*additional_value).into());
                                    }
                                } else if *cql_flavour == Flavour::Miabis {
                                    //for codes with no mappings in MIABIS the result will be empty
                                    string_array_with_workarounds.push(value);
                                }
                            }

                            let mut condition_humongous_string = "(".to_string();
                            let mut filter_humongous_string = "(".to_string();

                            for (index, string) in string_array_with_workarounds.iter().enumerate()
                            {
                                condition_humongous_string = condition_humongous_string
                                    + "("
                                    + condition_string.as_str()
                                    + ")";
                                condition_humongous_string =
                                    condition_humongous_string.replace("{{C}}", &escape(string));

                                filter_humongous_string =
                                    filter_humongous_string + "(" + filter_string.as_str() + ")";
                                filter_humongous_string =
                                    filter_humongous_string.replace("{{C}}", &escape(string));

                                // Only concatenate operator if it's not the last element
                                if index < string_array_with_workarounds.len() - 1 {
                                    condition_humongous_string += operator_str;
                                    filter_humongous_string += operator_str;
                                }
                            }
                            condition_string = condition_humongous_string + ")";

                            if !filter_string.is_empty() {
                                filter_string = filter_humongous_string + ")";
                            }
                        }
                        other => {
                            return Err(FocusError::AstOperatorValueMismatch(format!(
                                "Operator IN can only be used for string arrays, not for {:?}",
                                other
                            )));
                        }
                    }
                } // this becomes or of all
                ast::ConditionType::Equals => match condition.value {
                    ast::ConditionValue::String(string) => {
                        let operator_str = " or ";
                        let mut string_array_with_workarounds = if *cql_flavour == Flavour::Miabis {
                            //empty, codes get replaced
                            Default::default()
                        } else {
                            vec![string.clone()]
                        };
                        if let Some(additional_values) =
                            cql_flavour.get_code_workarounds().get(string.as_str())
                        {
                            for additional_value in additional_values {
                                string_array_with_workarounds.push((*additional_value).into());
                            }
                        } else if *cql_flavour == Flavour::Miabis {
                            //for codes with no mappings in MIABIS the result will be empty
                            string_array_with_workarounds.push(string);
                        }
                        let mut condition_humongous_string = "(".to_string();
                        let mut filter_humongous_string = "(".to_string();

                        for (index, string) in string_array_with_workarounds.iter().enumerate() {
                            condition_humongous_string =
                                condition_humongous_string + "(" + condition_string.as_str() + ")";
                            condition_humongous_string =
                                condition_humongous_string.replace("{{C}}", &escape(string));

                            filter_humongous_string =
                                filter_humongous_string + "(" + filter_string.as_str() + ")";
                            filter_humongous_string =
                                filter_humongous_string.replace("{{C}}", &escape(string));

                            // Only concatenate operator if it's not the last element
                            if index < string_array_with_workarounds.len() - 1 {
                                condition_humongous_string += operator_str;
                                filter_humongous_string += operator_str;
                            }
                        }
                        condition_string = condition_humongous_string + ")";

                        if !filter_string.is_empty() {
                            filter_string = filter_humongous_string + ")";
                        }
                    }
                    other => {
                        return Err(FocusError::AstOperatorValueMismatch(format!(
                            "Operator EQUALS can only be used for string arrays, not for {:?}",
                            other
                        )));
                    }
                },
                other => {
                    // won't get it from Lens yet
                    info!("Got this condition type which Lens is not programmed to send, ignoring: {:?}", other);
                }
            };

            retrieval_cond += condition_string.as_str();

            if !filter_cond.is_empty() && !filter_string.is_empty() {
                filter_cond += " and ";
            }

            filter_cond += filter_string.as_str(); // no condition needed, "" can be added with no change
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
                    cql_flavour,
                )?;

                // Only concatenate operator if it's not the last element
                if index < operation.children.len() - 1 {
                    retrieval_cond += operator_str;
                    if !filter_cond.is_empty()
                        && !filter_cond.ends_with(" or ")
                        && !filter_cond.ends_with(" and ")
                    {
                        filter_cond += operator_str;
                    }
                }
            }
            if let Some(pos) = filter_cond.rfind(')') {
                _ = filter_cond.split_off(pos + 1);
            }
        }
    }

    retrieval_cond += ")";

    *retrieval_criteria += retrieval_cond.as_str();

    if !filter_cond.is_empty() {
        *filter_criteria += "(";
        *filter_criteria += filter_cond.as_str();
        *filter_criteria += ")";

        *filter_criteria = filter_criteria.replace(")(", ") or (");
    }

    Ok(())
}

fn escape(value: &str) -> String {
    value
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("\'", "\\\'")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .replace("\n", "\\n")
}

fn formated_date_string(date: &str) -> Result<String, FocusError> {
    let datetime_str = date;

    let datetime_maybe: Result<DateTime<Utc>, _> = datetime_str.parse();

    let datetime_min: DateTime<Utc> = if let Ok(datetime) = datetime_maybe {
        datetime
    } else {
        let naive_date_maybe = NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d"); //FIXME remove once Lens2 behaves, only return the error

        if let Ok(naive_date) = naive_date_maybe {
            DateTime::<Utc>::from_naive_utc_and_offset(
                naive_date.and_time(NaiveTime::default()),
                Utc,
            )
        } else {
            return Err(FocusError::AstInvalidDateFormat(date.to_string()));
        }
    };

    Ok(format!("@{}", datetime_min.format("%Y-%m-%d")))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions;

    const AST: &str = r#"{"ast":{"operand":"AND","children":[{"key":"age","type":"EQUALS","value":5.0}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const MALE_OR_FEMALE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"},{"key":"gender","type":"EQUALS","system":"","value":"female"}]}]}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const ALL_GLIOMS: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"D43.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9383/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9384/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9394/1"},{"key":"59847-4","type":"EQUALS","system":"","value":"9421/1"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9382/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9391/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9400/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9424/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9425/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9450/3"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9440/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9441/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9442/3"}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C71.%"},{"key":"diagnosis","type":"EQUALS","system":"","value":"C72.%"}]},{"operand":"OR","children":[{"key":"59847-4","type":"EQUALS","system":"","value":"9381/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9382/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9401/3"},{"key":"59847-4","type":"EQUALS","system":"","value":"9451/3"}]}]}]}]}]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const AGE_AT_DIAGNOSIS_30_TO_70: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis_age_donor","type":"BETWEEN","system":"","value":{"min":30,"max":70}}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const AGE_AT_DIAGNOSIS_LOWER_THAN_70: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis_age_donor","type":"BETWEEN","system":"","value":{"min":0,"max":70}}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const C61_AND_MALE: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C61"}]},{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const ALL_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["male","other"]},{"children":[{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C25"},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C56"}],"de":"Diagnose ICD-10","en":"Diagnosis ICD-10","key":"diagnosis","operand":"OR"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-09-30T22:00:00.000Z"}},{"key":"bmi","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"body_weight","system":"","type":"BETWEEN","value":{"max":1100,"min":10}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Other fasting status"]},{"key":"smoking_status","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":10000,"min":100}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","blood-plasma","buffy-coat"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-10-03T22:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature-18to-35","temperature-60to-85"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const SOME_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["other","male"]},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C24"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":11,"min":1}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"bmi","system":"","type":"BETWEEN","value":{"max":111,"min":1}},{"key":"body_weight","system":"","type":"BETWEEN","value":{"max":1111,"min":110}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Not sober"]},{"key":"smoking_status","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":123,"min":1}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","tissue-other"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature2to10","temperatureGN"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const LENS2: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"},{"key":"gender","system":"","type":"EQUALS","value":"female"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"tissue-frozen"},{"key":"sample_kind","system":"","type":"EQUALS","value":"blood-serum"}],"operand":"OR"}],"operand":"AND"},{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"liquid-other"},{"key":"sample_kind","system":"","type":"EQUALS","value":"rna"},{"key":"sample_kind","system":"","type":"EQUALS","value":"urine"}],"operand":"OR"},{"children":[{"key":"storage_temperature","system":"","type":"EQUALS","value":"temperatureRoom"},{"key":"storage_temperature","system":"","type":"EQUALS","value":"four_degrees"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const EMPTY: &str =
        r#"{"ast":{"children":[],"operand":"OR"}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const EMPTY_OR: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[]},{"operand":"AND","children":[]}]},"id":"f1f59c3b-fbe6-4941-a718-c6656c96b70e"}"#;

    const LESS: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"diagnosis_age_donor","operand":"OR","children":[{"key":"diagnosis_age_donor","type":"BETWEEN","value":{"max":60}}]}]}]},"id":"6e914349-6cb1-4f84-b959-813c683f458d"}"#;

    const GREATER: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"diagnosis_age_donor","operand":"OR","children":[{"key":"diagnosis_age_donor","type":"BETWEEN","value":{"min":60}}]}]}]},"id":"6e914349-6cb1-4f84-b959-813c683f458d"}"#;

    const AFTER: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"sampling_date","operand":"OR","children":[{"key":"sampling_date","type":"BETWEEN","value":{"min":"2015-01-01"}}]}]}]},"id":"a0be5029-4c69-490e-b017-810308b0187a"}"#;

    const BEFORE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"sampling_date","operand":"OR","children":[{"key":"sampling_date","type":"BETWEEN","value":{"max":"2015-01-01"}}]}]}]},"id":"a0be5029-4c69-490e-b017-810308b0187a"}"#;

    const CURRENT: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]},{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C61"}]},{"operand":"OR","children":[{"key":"donor_age","type":"BETWEEN","system":"","value":{"min":10,"max":90}}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"sampling_date","type":"BETWEEN","system":"","value":{"min":"1900-01-01","max":"2024-10-25"}}]},{"operand":"OR","children":[{"key":"storage_temperature","type":"EQUALS","system":"","value":"temperature2to10"}]}]}]},"id":"53b4414e-75e4-401b-b794-20a2936e1be5"}"#;

    const VAFAN: &str = r#"{"ast":{"nodeType":"branch","operand":"OR","children":[{"nodeType":"branch","operand":"AND","children":[]}]},"id":"0b29f6d1-4e6a-4679-9212-3327e498b304__search__0b29f6d1-4e6a-4679-9212-3327e498b304"}"#;

    const QUOTE: &str = r#"{"ast": {"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C61\"'\\\n\r\t"}]},{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]}]}]}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    #[test]
    fn test_common() {
        // maybe nothing here
    }

    #[test]
    fn test_bbmri_quote() {
        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(QUOTE).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_quote.cql").to_string()
        );
    }

    #[test]
    fn test_bbmri() {
        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(MALE_OR_FEMALE).unwrap(),
                Flavour::Bbmri
            )
            .unwrap(),
            include_str!("../resources/test/result_male_or_female.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(AGE_AT_DIAGNOSIS_30_TO_70).unwrap(),
                Flavour::Bbmri
            )
            .unwrap(),
            include_str!("../resources/test/result_age_at_diagnosis_30_to_70.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(AGE_AT_DIAGNOSIS_LOWER_THAN_70).unwrap(),
                Flavour::Bbmri
            )
            .unwrap(),
            include_str!("../resources/test/result_age_at_diagnosis_lower_than_70.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(C61_AND_MALE).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_c61_and_male.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(ALL_GBN).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_all_gbn.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(SOME_GBN).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_some_gbn.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(LENS2).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_lens2.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(EMPTY).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_empty.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(EMPTY_OR).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_empty.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(LESS).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_less.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(GREATER).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_greater.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(AFTER).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_after.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(BEFORE).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_before.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(VAFAN).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_empty.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(CURRENT).unwrap(), Flavour::Bbmri).unwrap(),
            include_str!("../resources/test/result_current.cql").to_string()
        );
    }

    const DIAGNOSIS_C30: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"diagnosis","operand":"OR","children":[{"operand":"OR","key":"diagnosis","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C30"},{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C30.1"},{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C30.0"}]}]}]}]},"id":"54bd1d51-aa35-4153-b49e-56753774fa2d"}"#;

    const YEAR_OF_DIAGNOSIS_2000_TO_2010: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"year_of_diagnosis","operand":"OR","children":[{"key":"year_of_diagnosis","type":"BETWEEN","system":"","value":{"min":2000,"max":2010}}]}]}]},"id":"c9024169-dfcf-4915-ac06-1edd090054f4"}"#;

    const BODY_SITE_LEFT: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"bodySite","operand":"OR","children":[{"key":"bodySite","type":"EQUALS","system":"http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SeitenlokalisationCS","value":"L"}]}]}]},"id":"c3481b0c-4807-4d54-b7c9-498426f165c2"}"#;

    const GRADING_LOW_GRADE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"grading","operand":"OR","children":[{"key":"grading","type":"EQUALS","system":"http://dktk.dkfz.de/fhir/onco/core/CodeSystem/GradingCS","value":"L"}]}]}]},"id":"34fbeac2-6685-4e48-b531-372d8aa8589f"}"#;

    const TNM_T_2: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"TNM-T","operand":"OR","children":[{"key":"TNM-T","type":"EQUALS","system":"","value":"2"}]}]}]},"id":"3cc4f25a-e086-4a09-afa1-b07ae77d7144"}"#;

    const ANALYSIS_METHOD_WGS: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"analysis_method","operand":"OR","children":[{"key":"analysis_method","type":"EQUALS","system":"http://dktk.dkfz.de/fhir/onco/core/CodeSystem/AnalysemethodeCS","value":"wgs"}]}]}]},"id":"9d1a8fd6-4b1e-4a2f-9d3e-6c1f0b7a5e21"}"#;

    const VARIANT_TYPE_MISSENSE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"observationMolecularMarkerVariantType","operand":"OR","children":[{"key":"observationMolecularMarkerVariantType","type":"EQUALS","system":"http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VariantenTypCS","value":"nonsynonymous-snv"}]}]}]},"id":"7c2e5a90-3f4b-4d1a-8e77-2b9c6d0f1a34"}"#;

    const DNA_CHANGE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"observationMolecularMarkerDNAchange","operand":"OR","children":[{"key":"observationMolecularMarkerDNAchange","type":"EQUALS","system":"","value":"c.2147C>T"}]}]}]},"id":"1f5b8c72-9d3e-4a06-b2c8-5e7f4a1d9b60"}"#;

    const SAMPLE_KIND_FFPE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"sample_kind","operand":"OR","children":[{"key":"Gewebe FFPE","operand":"AND","children":[{"operand":"OR","children":[{"key":"sample_kind","type":"EQUALS","system":"","value":"tumor-tissue-ffpe"},{"key":"histology","type":"EQUALS","system":"","value":"tumor-tissue-ffpe"},{"key":"sample_kind","type":"EQUALS","system":"","value":"tissue-ffpe"},{"key":"sample_kind","type":"EQUALS","system":"","value":"normal-tissue-ffpe"},{"key":"sample_kind","type":"EQUALS","system":"","value":"other-tissue-ffpe"}]}]}]}]}]},"id":"bd5112af-e712-4091-9c94-892c4e667a29"}"#;

    #[test]
    fn test_dktk() {
        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(DIAGNOSIS_C30).unwrap(), Flavour::Dktk).unwrap(),
            include_str!("../resources/test/result_diagnosis_c30.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(YEAR_OF_DIAGNOSIS_2000_TO_2010).unwrap(),
                Flavour::Dktk
            )
            .unwrap(),
            include_str!("../resources/test/result_year_of_diagnosis_2000_to_2010.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(BODY_SITE_LEFT).unwrap(), Flavour::Dktk).unwrap(),
            include_str!("../resources/test/result_body_site_left.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(GRADING_LOW_GRADE).unwrap(),
                Flavour::Dktk
            )
            .unwrap(),
            include_str!("../resources/test/result_grading_low_grade.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(TNM_T_2).unwrap(), Flavour::Dktk).unwrap(),
            include_str!("../resources/test/result_tnm_t_2.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(ANALYSIS_METHOD_WGS).unwrap(),
                Flavour::Dktk
            )
            .unwrap(),
            include_str!("../resources/test/result_analysis_method_wgs.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(VARIANT_TYPE_MISSENSE).unwrap(),
                Flavour::Dktk
            )
            .unwrap(),
            include_str!("../resources/test/result_variant_type_missense.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(DNA_CHANGE).unwrap(), Flavour::Dktk).unwrap(),
            include_str!("../resources/test/result_dna_change.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(
                serde_json::from_str(SAMPLE_KIND_FFPE).unwrap(),
                Flavour::Dktk
            )
            .unwrap(),
            include_str!("../resources/test/result_sample_kind_ffpe.cql").to_string()
        );
    }

    const CCE_MALE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"gender","operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]}]}]},"id":"8bb53643-fe28-4556-a808-528a4274bea5"}"#;
    const CCE_ALIVE: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"key":"vitalStatusCS","operand":"OR","children":[{"key":"vitalStatusCS","type":"EQUALS","system":"https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VitalStatusCS","value":"alive"}]}]}]},"id":"ba71c2d5-feb1-4649-800e-aaaac2e4bcb0"}"#;

    const CCE_VITAL_STATUS_URL: &str =
        "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VitalStatusCS";
    const CCE_SAMPLE_MATERIAL_TYPE_URL: &str =
        "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SampleMaterialType";
    const CCE_SYST_THERAPY_TYPE_URL: &str =
        "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SYSTTherapyTypeCS";

    #[test]
    fn test_cce_empty() {
        let generated_cql =
            generate_cql(serde_json::from_str(EMPTY).unwrap(), Flavour::Cce).unwrap();
        pretty_assertions::assert_eq!(generated_cql.contains(CCE_VITAL_STATUS_URL), true);
        pretty_assertions::assert_eq!(generated_cql.contains(CCE_SAMPLE_MATERIAL_TYPE_URL), true);
        pretty_assertions::assert_eq!(generated_cql.contains(CCE_SYST_THERAPY_TYPE_URL), true);

        pretty_assertions::assert_eq!(
            generated_cql,
            include_str!("../resources/test/result_cce_base.cql").to_string()
        );
    }

    #[test]
    fn test_cce_male() {
        let expected = r#"Patient.gender = 'male'"#;
        let generated_cql =
            generate_cql(serde_json::from_str(CCE_MALE).unwrap(), Flavour::Cce).unwrap();
        pretty_assertions::assert_eq!(generated_cql.contains(expected), true);
    }

    // #[test]
    // fn test_cce_alive() {
    //     let expected = r#"Patient.gender = 'male'"#;
    //     let generated_cql = generate_cql(serde_json::from_str(CCE_ALIVE).unwrap(), Project::Cce);
    //     println!("generated cql query: {:?}", generated_cql);

    //     // pretty_assertions::assert_eq!(generated_cql.contains(expected), true);
    //     pretty_assertions::assert_eq!(false, true);
    // }

    const MIABIS_DIAGNOSIS: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"","value":"C61"}]}]}]},"id":"miabis-d1"}"#;

    const MIABIS_SAMPLE_EQUALS_WHOLE_BLOOD: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"sample_kind","type":"EQUALS","system":"","value":"whole-blood"}]}]}]},"id":"miabis-s1"}"#;

    const MIABIS_DATE_OF_DIAGNOSIS: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"date_of_diagnosis","type":"BETWEEN","system":"","value":{"min":"2020-01-01","max":"2021-01-01"}}]}]}]},"id":"miabis-dd1"}"#;

    const MIABIS_DIAGNOSIS_AGE_DONOR: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"diagnosis_age_donor","type":"BETWEEN","system":"","value":{"min":30,"max":70}}]}]}]},"id":"miabis-da1"}"#;

    const MIABIS_SAMPLE_IN: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"sample_kind","type":"IN","system":"","value":["blood-plasma","tissue-ffpe"]}]}]}]},"id":"miabis-s2"}"#;

    const MIABIS_SAMPLE_LIQUID_OTHER: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"sample_kind","type":"IN","system":"","value":["liquid-other"]}]}]}]},"id":"miabis-s3"}"#;

    const MIABIS_TEMP_EQUALS_ROOM: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"storage_temperature","type":"EQUALS","system":"","value":"temperatureRoom"}]}]}]},"id":"miabis-t1"}"#;

    const MIABIS_TEMP_EQUALS_FOUR: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"storage_temperature","type":"EQUALS","system":"","value":"four_degrees"}]}]}]},"id":"miabis-t2"}"#;

    const MIABIS_TEMP_IN: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"storage_temperature","type":"IN","system":"","value":["temperature2to10","temperatureGN"]}]}]}]},"id":"miabis-t3"}"#;

    const MIABIS_TEMP_UNCHARTED: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"storage_temperature","type":"EQUALS","system":"","value":"storage_temperature_uncharted"}]}]}]},"id":"miabis-t4"}"#;

    #[test]
    fn test_miabis_empty() {
        let cql = generate_cql(serde_json::from_str(EMPTY).unwrap(), Flavour::Miabis).unwrap();
        assert!(cql.contains("http://hl7.org/fhir/sid/icd-10"));
        assert!(
            cql.contains("https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs")
        );
    }

    #[test]
    fn test_miabis_gender() {
        let cql = generate_cql(
            serde_json::from_str(MALE_OR_FEMALE).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains("Patient.gender = 'male'"));
        assert!(cql.contains("Patient.gender = 'female'"));
    }

    #[test]
    fn test_miabis_diagnosis() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_DIAGNOSIS).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(
            "(exists from [Observation] O where (O.value as CodeableConcept).coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code contains 'C61')"
        ));
        assert!(cql.contains("(exists[Condition: Code 'C61' from icd10])"));
        assert!(cql.contains("http://hl7.org/fhir/sid/icd-10"));
        assert!(!cql.contains("http://fhir.de/CodeSystem/dimdi/icd-10-gm"));
    }

    #[test]
    fn test_miabis_diagnosis_stratifier_reads_observation() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_DIAGNOSIS).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains("define Diagnosis:\n    if InInitialPopulation then [Observation]"));
        assert!(cql.contains(
            "define function DiagnosisCode(observation FHIR.Observation):\n    (observation.value as CodeableConcept)"
        ));
    }

    #[test]
    fn test_miabis_date_of_diagnosis() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_DATE_OF_DIAGNOSIS).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(
            cql.contains("(exists from [Observation] O where FHIRHelpers.ToDateTime(O.effective)")
        );
        assert!(cql.contains("(exists from [Condition] C where FHIRHelpers.ToDateTime(C.onset)"));
    }

    #[test]
    fn test_miabis_diagnosis_age_donor() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_DIAGNOSIS_AGE_DONOR).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(
            "(exists from [Observation] O where AgeInYearsAt(FHIRHelpers.ToDateTime(O.effective))"
        ));
        assert!(cql.contains(
            "(exists from [Condition] C where AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset))"
        ));
    }

    #[test]
    fn test_miabis_sample_kind() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_SAMPLE_EQUALS_WHOLE_BLOOD).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'WholeBlood')"#));
        assert!(!cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'whole-blood')"#));
        assert!(
            cql.contains("https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs")
        );
    }

    #[test]
    fn test_miabis_storage_temperature() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_TEMP_EQUALS_ROOM).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(!cql.contains("'temperatureRoom'"));
        assert!(cql.contains(r#"(E.value as CodeableConcept).coding.code contains 'RT')"#));
        assert!(cql.contains("S.processing"));
        assert!(cql.contains(
            "https://fhir.bbmri-eric.eu/StructureDefinition/miabis-sample-storage-temperature-extension"
        ));
    }

    #[test]
    fn test_miabis_sample_type_workarounds() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_SAMPLE_EQUALS_WHOLE_BLOOD).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'WholeBlood')"#));
        assert!(!cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'whole-blood')"#));

        let cql = generate_cql(
            serde_json::from_str(MIABIS_SAMPLE_IN).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'Plasma')"#));
        assert!(!cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'blood-plasma')"#));
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'TissueFixed')"#));
        assert!(!cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'tissue-ffpe')"#));

        let cql = generate_cql(
            serde_json::from_str(MIABIS_SAMPLE_LIQUID_OTHER).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'LiquidBiopsy')"#));
        assert!(cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'Sputum')"#));
        assert!(!cql.contains(r#"(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains 'liquid-other')"#));
    }

    #[test]
    fn test_miabis_storage_temperature_workarounds() {
        let cql = generate_cql(
            serde_json::from_str(MIABIS_TEMP_EQUALS_ROOM).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(!cql.contains("'temperatureRoom'"));
        assert!(cql.contains("'RT'"));

        let cql = generate_cql(
            serde_json::from_str(MIABIS_TEMP_EQUALS_FOUR).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(!cql.contains("'four_degrees'"));
        assert!(cql.contains("'2to10'"));

        let cql = generate_cql(
            serde_json::from_str(MIABIS_TEMP_IN).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(!cql.contains("'temperature2to10'"));
        assert!(cql.contains("'2to10'"));
        assert!(!cql.contains("'temperatureGN'"));
        assert!(cql.contains("'LN'"));

        let cql = generate_cql(
            serde_json::from_str(MIABIS_TEMP_UNCHARTED).unwrap(),
            Flavour::Miabis,
        )
        .unwrap();
        assert!(cql.contains("coding.code contains 'storage_temperature_uncharted'"));
        for miabis_code in &[
            "'RT'",
            "'2to10'",
            "'-18to-35'",
            "'-60to-85'",
            "'LN'",
            "'Other'",
        ] {
            assert!(!cql.contains(miabis_code));
        }
    }
}
