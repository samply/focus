use crate::ast;
use crate::errors::FocusError;

use chrono::offset::Utc;
use chrono::DateTime;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
enum CriterionRole {
    Query,
    Filter,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Project {
    Bbmri,
    Dktk,
}

static CODE_LISTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    //code lists with their names
    [
        ("icd10", "http://hl7.org/fhir/sid/icd-10"),
        ("icd10gm", "http://fhir.de/CodeSystem/dimdi/icd-10-gm"),
        ("loinc", "http://loinc.org"),
        (
            "SampleMaterialType",
            "https://fhir.bbmri.de/CodeSystem/SampleMaterialType",
        ),
        (
            "StorageTemperature",
            "https://fhir.bbmri.de/CodeSystem/StorageTemperature",
        ),
        (
            "FastingStatus",
            "http://terminology.hl7.org/CodeSystem/v2-0916",
        ),
        (
            "SmokingStatus",
            "http://hl7.org/fhir/uv/ips/ValueSet/current-smoking-status-uv-ips",
        ),
    ]
    .into()
});

static OBSERVATION_LOINC_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    [
        ("body_weight", "29463-7"),
        ("bmi", "39156-5"),
        ("smoking_status", "72166-2"),
    ]
    .into()
});

static CRITERION_CODE_LISTS: Lazy<HashMap<(&str, Project), Vec<&str>>> = Lazy::new(|| {
    // code lists needed depending on the criteria selected
    [
        (("diagnosis", Project::Bbmri), vec!["icd10", "icd10gm"]),
        (("body_weight", Project::Bbmri), vec!["loinc"]),
        (("bmi", Project::Bbmri), vec!["loinc"]),
        (("smoking_status", Project::Bbmri), vec!["loinc"]),
        (("sample_kind", Project::Bbmri), vec!["SampleMaterialType"]),
        (
            ("storage_temperature", Project::Bbmri),
            vec!["StorageTemperature"],
        ),
        (("fasting_status", Project::Bbmri), vec!["FastingStatus"]),
    ]
    .into()
});

static CQL_SNIPPETS: Lazy<HashMap<(&str, CriterionRole, Project), &str>> = Lazy::new(|| {
    // CQL snippets depending on the criteria
    [
        (("gender", CriterionRole::Query, Project::Bbmri), "Patient.gender = '{{C}}'"),
        (
            ("diagnosis", CriterionRole::Query, Project::Bbmri),
            " ((exists[Condition: Code '{{C}}' from {{A1}}]) or (exists[Condition: Code '{{C}}' from {{A2}}])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains '{{C}}')) ",
        ),
        (("diagnosis_old", CriterionRole::Query, Project::Bbmri), " exists [Condition: Code '{{C}}' from {{A1}}] "),
        (
            ("date_of_diagnosis", CriterionRole::Query, Project::Bbmri),
            " exists from [Condition] C\nwhere FHIRHelpers.ToDateTime(C.onset) between {{D1}} and {{D2}} ",
        ),
        (
            ("diagnosis_age_donor", CriterionRole::Query, Project::Bbmri),
            " exists from [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling({{D1}}) and Ceiling({{D2}}) ",
        ),
        (("donor_age", CriterionRole::Query, Project::Bbmri), " AgeInYears() between Ceiling({{D1}}) and Ceiling({{D2}}) "),
        (
            ("observationRange", CriterionRole::Query, Project::Bbmri),
            " exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value between {{D1}} and {{D2}} ",
        ),
        (
            ("body_weight", CriterionRole::Query, Project::Bbmri),
            " exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg' and (O.value as Quantity) > {{D2}} 'kg') ",
        ),
        (
            ("bmi", CriterionRole::Query, Project::Bbmri),
            " exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg/m2' and (O.value as Quantity) > {{D2}} 'kg/m2') ",
        ),
        (("sample_kind", CriterionRole::Query, Project::Bbmri), " exists [Specimen: Code '{{C}}' from {{A1}}] "),
        (("sample_kind", CriterionRole::Filter, Project::Bbmri), " (S.type.coding.code contains '{{C}}') "),

        (
            ("storage_temperature", CriterionRole::Filter, Project::Bbmri),
            " (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains '{{C}}') ",
        ),
        (
            ("sampling_date", CriterionRole::Filter, Project::Bbmri),
            " (FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}}) ",
        ),
        (
            ("fasting_status", CriterionRole::Filter, Project::Bbmri),
            " (S.collection.fastingStatus.coding.code contains '{{C}}') ",
        ),
        (
            ("sampling_date", CriterionRole::Query, Project::Bbmri),
            " exists from [Specimen] S\nwhere FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}} ",
        ),
        (
            ("fasting_status", CriterionRole::Query, Project::Bbmri),   
            " exists from [Specimen] S\nwhere S.collection.fastingStatus.coding.code contains '{{C}}' ",
        ),
        (
            ("storage_temperature", CriterionRole::Query, Project::Bbmri), 
            " exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code '{{C}}' from {{A1}}) ",
        ),
        (
            ("smoking_status", CriterionRole::Query, Project::Bbmri), 
            " exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}' ",
        ),
    ]
    .into()
});

pub fn bbmri(ast: ast::Ast) -> Result<String, FocusError> {
    let mut retrieval_criteria: String = "(".to_string(); // main selection criteria (Patient)

    let mut filter_criteria: String = " where (".to_string(); // criteria for filtering specimens

    let mut code_systems: HashSet<&str> = HashSet::new(); // code lists needed depending on the criteria
    code_systems.insert("icd10"); //for diagnosis stratifier
    code_systems.insert("SampleMaterialType"); //for sample type stratifier

    let mut lists: String = "".to_string(); // needed code lists, defined

    let mut cql: String = include_str!("../resources/template_bbmri.cql").to_string();

    let operator_str = match ast.ast.operand {
        ast::Operand::And => " and ",
        ast::Operand::Or => " or ",
    };

    for (index, grandchild) in ast.ast.children.iter().enumerate() {
        process(
            grandchild.clone(),
            &mut retrieval_criteria,
            &mut filter_criteria,
            &mut code_systems,
            Project::Bbmri,
        )?;

        // Only concatenate operator if it's not the last element
        if index < ast.ast.children.len() - 1 {
            retrieval_criteria += operator_str;
        }
    }

    retrieval_criteria += ")";
    filter_criteria += ")";

    for code_system in code_systems {
        lists += format!(
            "codesystem {}: '{}' \n",
            code_system,
            CODE_LISTS.get(code_system).unwrap_or(&(""))
        )
        .as_str();
    }

    cql = cql
        .replace("{{lists}}", lists.as_str())
        .replace("{{filter_criteria}}", filter_criteria.as_str());

    if retrieval_criteria != *"()" {
        // no criteria selected
        cql = cql.replace("{{retrieval_criteria}}", retrieval_criteria.as_str());
    } else {
        cql = cql.replace("{{retrieval_criteria}}", "true");
    }

    if filter_criteria != *" where ()" {
        // no criteria selected
        cql = cql.replace("{{retrieval_criteria}}", filter_criteria.as_str());
    } else {
        cql = cql.replace("{{retrieval_criteria}}", "");
    }

    Ok(cql)
}

pub fn process(
    child: ast::Child,
    retrieval_criteria: &mut String,
    filter_criteria: &mut String,
    code_systems: &mut HashSet<&str>,
    project: Project,
) -> Result<(), FocusError> {
    let mut retrieval_cond: String = "(".to_string();
    let mut filter_cond: String = "".to_string();

    match child {
        ast::Child::Condition(condition) => {
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
                        code_systems.insert(code_list);
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
                        match condition.value {
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

                        match condition.value {
                            ast::ConditionValue::StringArray(string_array) => {
                                let mut condition_humongous_string = " (".to_string();
                                let mut filter_humongous_string = " (".to_string();

                                for (index, string) in string_array.iter().enumerate() {
                                    condition_humongous_string = condition_humongous_string
                                        + " ("
                                        + condition_string.as_str()
                                        + ") ";
                                    condition_humongous_string = condition_humongous_string
                                        .replace("{{C}}", string.as_str());

                                    filter_humongous_string = filter_humongous_string
                                        + " ("
                                        + filter_string.as_str()
                                        + ") ";
                                    filter_humongous_string =
                                        filter_humongous_string.replace("{{C}}", string.as_str());

                                    // Only concatenate operator if it's not the last element
                                    if index < string_array.len() - 1 {
                                        condition_humongous_string += operator_str;
                                        filter_humongous_string += operator_str;
                                    }
                                }
                                condition_string = condition_humongous_string + " )";

                                if filter_string != "" {
                                    filter_string = filter_humongous_string + " )";
                                }
                            }
                            _ => {
                                return Err(FocusError::AstOperatorValueMismatch());
                            }
                        }
                    } // this becomes or of all
                    ast::ConditionType::Equals => match condition.value {
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

                retrieval_cond += condition_string.as_str();
                filter_cond += filter_string.as_str(); // no condition needed, "" can be added with no change
            } else {
                return Err(FocusError::AstUnknownCriterion(
                    condition_key_trans.to_string(),
                ));
            }
            if filter_cond != "" {
                filter_cond += " ";
            }
            retrieval_cond += " ";
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
                    if filter_cond != "" {
                        filter_cond += operator_str;
                        dbg!(filter_cond.clone());
                    }
                }
            }
        }
    }

    retrieval_cond += ")";

    *retrieval_criteria += retrieval_cond.as_str();

    if filter_cond != "" { 
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
    fn test_just_print() {
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

        println!(
            "{:?}",
            bbmri(serde_json::from_str(ALL_GBN).expect("Failed to deserialize JSON"))
        );

        println!();

        println!(
            "{:?}",
            bbmri(serde_json::from_str(SOME_GBN).expect("Failed to deserialize JSON"))
        );

        println!();

        println!(
            "{:?}",
            bbmri(serde_json::from_str(LENS2).expect("Failed to deserialize JSON"))
        );

        println!();

        println!(
            "{:?}",
            bbmri(serde_json::from_str(EMPTY).expect("Failed to deserialize JSON"))
        );
    }
}
