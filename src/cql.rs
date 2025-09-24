use crate::ast;
use crate::errors::FocusError;
use crate::projects::{
    CriterionRole, BODY, CODE_LISTS, CQL_SNIPPETS, CQL_TEMPLATE, CRITERION_CODE_LISTS,
    MANDATORY_CODE_SYSTEMS, OBSERVATION_LOINC_CODE, SAMPLE_TYPE_WORKAROUNDS,
};

use base64::{prelude::BASE64_STANDARD as BASE64, Engine as _};
use chrono::offset::Utc;
use chrono::{DateTime, NaiveDate, NaiveTime};
use indexmap::set::IndexSet;
use tracing::info;
use uuid::Uuid;

pub fn generate_body(ast: ast::Ast) -> Result<String, FocusError> {
    Ok(BODY
        .to_string()
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
            BASE64.encode(generate_cql(ast)?).as_str(),
        ))
}

fn generate_cql(ast: ast::Ast) -> Result<String, FocusError> {
    let mut retrieval_criteria: String = String::new(); // main selection criteria (Patient)

    let mut filter_criteria: String = String::new(); // criteria for filtering specimens

    let mut lists: String = String::new(); // needed code lists, defined

    let mut cql = CQL_TEMPLATE.clone().to_string();

    let operator_str = match ast.ast.operand {
        ast::Operand::And => " and ",
        ast::Operand::Or => " or ",
    };

    let mut mandatory_codes = MANDATORY_CODE_SYSTEMS.clone();

    for (index, grandchild) in ast.ast.children.iter().enumerate() {
        process(
            grandchild.clone(),
            &mut retrieval_criteria,
            &mut filter_criteria,
            &mut mandatory_codes,
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

    cql = cql.replace("{{lists}}", lists.as_str());

    if retrieval_criteria.is_empty()
        || retrieval_criteria
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
) -> Result<(), FocusError> {
    let mut retrieval_cond: String = "(".to_string();
    let mut filter_cond: String = String::new();

    match child {
        ast::Child::Condition(condition) => {
            let condition_key_trans = condition.key.as_str();

            let condition_snippet = CQL_SNIPPETS.get(&(condition_key_trans, CriterionRole::Query));

            let Some(snippet) = condition_snippet else {
                return Err(FocusError::AstUnknownCriterion(
                    condition_key_trans.to_string(),
                ));
            };
            let mut condition_string = (*snippet).to_string();
            let mut filter_string: String = String::new();

            let filter_snippet = CQL_SNIPPETS.get(&(condition_key_trans, CriterionRole::Filter));

            let code_lists_option = CRITERION_CODE_LISTS.get(&(condition_key_trans));
            if let Some(code_lists_vec) = code_lists_option {
                for (index, code_list) in code_lists_vec.iter().enumerate() {
                    code_systems.insert(code_list);
                    let placeholder = format!("{{{{A{}}}}}", (index + 1)); //to keep compatibility with snippets in typescript
                    condition_string = condition_string.replace(placeholder.as_str(), code_list);
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
                ast::ConditionType::Between => {
                    // both min and max values stated
                    match condition.value {
                        ast::ConditionValue::DateRange(date_range) => {
                            let datetime_str_min = date_range.min.as_str();

                            let datetime_min_maybe: Result<DateTime<Utc>, _> =
                                datetime_str_min.parse();

                            let datetime_min: DateTime<Utc> = if let Ok(datetime) =
                                datetime_min_maybe
                            {
                                datetime
                            } else {
                                let naive_date_maybe =
                                    NaiveDate::parse_from_str(datetime_str_min, "%Y-%m-%d"); //FIXME remove once Lens2 behaves, only return the error

                                if let Ok(naive_date) = naive_date_maybe {
                                    DateTime::<Utc>::from_naive_utc_and_offset(
                                        naive_date.and_time(NaiveTime::default()),
                                        Utc,
                                    )
                                } else {
                                    return Err(FocusError::AstInvalidDateFormat(date_range.min));
                                }
                            };

                            let date_str_min = format!("@{}", datetime_min.format("%Y-%m-%d"));

                            condition_string =
                                condition_string.replace("{{D1}}", date_str_min.as_str());
                            filter_string = filter_string.replace("{{D1}}", date_str_min.as_str());
                            // no condition needed, "" stays ""

                            let datetime_str_max = date_range.max.as_str();
                            let datetime_max_maybe: Result<DateTime<Utc>, _> =
                                datetime_str_max.parse();

                            let datetime_max: DateTime<Utc> = if let Ok(datetime) =
                                datetime_max_maybe
                            {
                                datetime
                            } else {
                                let naive_date_maybe =
                                    NaiveDate::parse_from_str(datetime_str_max, "%Y-%m-%d"); //FIXME remove once Lens2 behaves, only return the error

                                if let Ok(naive_date) = naive_date_maybe {
                                    DateTime::<Utc>::from_naive_utc_and_offset(
                                        naive_date.and_time(NaiveTime::default()),
                                        Utc,
                                    )
                                } else {
                                    return Err(FocusError::AstInvalidDateFormat(date_range.max));
                                }
                            };
                            let date_str_max = format!("@{}", datetime_max.format("%Y-%m-%d"));

                            condition_string =
                                condition_string.replace("{{D2}}", date_str_max.as_str());
                            filter_string = filter_string.replace("{{D2}}", date_str_max.as_str());

                            // no condition needed, "" stays ""
                        }
                        ast::ConditionValue::NumRange(num_range) => {
                            condition_string = condition_string
                                .replace("{{D1}}", num_range.min.to_string().as_str());
                            condition_string = condition_string
                                .replace("{{D2}}", num_range.max.to_string().as_str());
                            filter_string =
                                filter_string.replace("{{D1}}", num_range.min.to_string().as_str()); // no condition needed, "" stays ""
                            filter_string =
                                filter_string.replace("{{D2}}", num_range.max.to_string().as_str());

                            // no condition needed, "" stays ""
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
                            let mut string_array_with_workarounds = string_array.clone();
                            for value in string_array {
                                if let Some(additional_values) =
                                    SAMPLE_TYPE_WORKAROUNDS.get(value.as_str())
                                {
                                    for additional_value in additional_values {
                                        string_array_with_workarounds
                                            .push((*additional_value).into());
                                    }
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
                                    condition_humongous_string.replace("{{C}}", string.as_str());

                                filter_humongous_string =
                                    filter_humongous_string + "(" + filter_string.as_str() + ")";
                                filter_humongous_string =
                                    filter_humongous_string.replace("{{C}}", string.as_str());

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
                        let mut string_array_with_workarounds = vec![string.clone()];
                        if let Some(additional_values) =
                            SAMPLE_TYPE_WORKAROUNDS.get(string.as_str())
                        {
                            for additional_value in additional_values {
                                string_array_with_workarounds.push((*additional_value).into());
                            }
                        }
                        let mut condition_humongous_string = "(".to_string();
                        let mut filter_humongous_string = "(".to_string();

                        for (index, string) in string_array_with_workarounds.iter().enumerate() {
                            condition_humongous_string =
                                condition_humongous_string + "(" + condition_string.as_str() + ")";
                            condition_humongous_string =
                                condition_humongous_string.replace("{{C}}", string.as_str());

                            filter_humongous_string =
                                filter_humongous_string + "(" + filter_string.as_str() + ")";
                            filter_humongous_string =
                                filter_humongous_string.replace("{{C}}", string.as_str());

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

    const CURRENT: &str = r#"{"ast":{"operand":"OR","children":[{"operand":"AND","children":[{"operand":"OR","children":[{"key":"gender","type":"EQUALS","system":"","value":"male"}]},{"operand":"OR","children":[{"key":"diagnosis","type":"EQUALS","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","value":"C61"}]},{"operand":"OR","children":[{"key":"donor_age","type":"BETWEEN","system":"","value":{"min":10,"max":90}}]}]},{"operand":"AND","children":[{"operand":"OR","children":[{"key":"sampling_date","type":"BETWEEN","system":"","value":{"min":"1900-01-01","max":"2024-10-25"}}]},{"operand":"OR","children":[{"key":"storage_temperature","type":"EQUALS","system":"","value":"temperature2to10"}]}]}]},"id":"53b4414e-75e4-401b-b794-20a2936e1be5"}"#;

    const VAFAN: &str = r#"{"ast":{"nodeType":"branch","operand":"OR","children":[{"nodeType":"branch","operand":"AND","children":[]}]},"id":"0b29f6d1-4e6a-4679-9212-3327e498b304__search__0b29f6d1-4e6a-4679-9212-3327e498b304"}"#;

    #[test]
    fn test_common() {
        // maybe nothing here
    }

    #[test]
    #[cfg(feature = "bbmri")]
    fn test_bbmri() {
        use crate::projects::{self, bbmri::Bbmri};

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(MALE_OR_FEMALE).unwrap()).unwrap(),
            include_str!("../resources/test/result_male_or_female.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(AGE_AT_DIAGNOSIS_30_TO_70).unwrap()).unwrap(),
            include_str!("../resources/test/result_age_at_diagnosis_30_to_70.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(AGE_AT_DIAGNOSIS_LOWER_THAN_70).unwrap()).unwrap(),
            include_str!("../resources/test/result_age_at_diagnosis_lower_than_70.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(C61_AND_MALE).unwrap()).unwrap(),
            include_str!("../resources/test/result_c61_and_male.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(ALL_GBN).unwrap()).unwrap(),
            include_str!("../resources/test/result_all_gbn.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(SOME_GBN).unwrap()).unwrap(),
            include_str!("../resources/test/result_some_gbn.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(LENS2).unwrap()).unwrap(),
            include_str!("../resources/test/result_lens2.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(EMPTY).unwrap()).unwrap(),
            include_str!("../resources/test/result_empty.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(VAFAN).unwrap()).unwrap(),
            include_str!("../resources/test/result_empty.cql").to_string()
        );

        pretty_assertions::assert_eq!(
            generate_cql(serde_json::from_str(CURRENT).unwrap()).unwrap(),
            include_str!("../resources/test/result_current.cql").to_string()
        );
    }

    #[test]
    #[cfg(feature = "dktk")]
    fn test_dktk() {
        //use crate::projects::{self, dktk::Dktk};

        // TODO Implement DKTK CQL generation and create files with results

        //pretty_assertions::assert_eq!(generate_cql(serde_json::from_str(AST).unwrap(), Bbmri).unwrap(), include_str!("../resources/test/result_ast.cql").to_string());

        //pretty_assertions::assert_eq!(generate_cql(serde_json::from_str(ALL_GLIOMS).unwrap(), Bbmri).unwrap(), include_str!("../resources/test/result_all_glioms.cql").to_string());
    }
}
