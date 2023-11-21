use crate::ast;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::HashSet;

enum CriterionRole {
    Query,
    Filter,
}

const OBSERVATION_BMI: &str = "39156-5";
const OBSERVATION_BODY_WEIGHT: &str = "29463-7";

static ALIAS_BBMRI: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let map = [
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
    .into();

    map
});

static CQL_TEMPLATE_BBMRI: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let map = [
        ("gender", "Patient.gender"),
        (
            "conditionSampleDiagnosis",
            "((exists[Condition: Code '{{C}}' from {{A1}}]) or (exists[Condition: Code '{{C}}' from {{A2}}])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains '{{C}}'))",
        ),
        ("conditionValue", "exists [Condition: Code '{{C}}' from {{A1}}]"),
        (
            "conditionRangeDate",
            "exists from [Condition] C\nwhere FHIRHelpers.ToDateTime(C.onset) between {{D1}} and {{D2}}",
        ),
        (
            "conditionRangeAge",
            "exists from [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling({{D1}}) and Ceiling({{D2}})",
        ),
        ("age", "AgeInYears() between Ceiling({{D1}}) and Ceiling({{D2}})"),
        (
            "observation",
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}'",
        ),
        (
            "observationRange",
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value between {{D1}} and {{D2}}",
        ),
        (
            "observationBodyWeight",
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg' and (O.value as Quantity) > {{D2}} 'kg')",
        ),
        (
            "observationBMI",
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg/m2' and (O.value as Quantity) > {{D2}} 'kg/m2')",
        ),
        ("hasSpecimen", "exists [Specimen]"),
        ("specimen", "exists [Specimen: Code '{{C}}' from {{A1}}]"),
        ("retrieveSpecimenByType", "(S.type.coding.code contains '{{C}}')"),
        (
            "retrieveSpecimenByTemperature",
            "(S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains '{{C}}')",
        ),
        (
            "retrieveSpecimenBySamplingDate",
            "(FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}})",
        ),
        (
            "retrieveSpecimenByFastingStatus",
            "(S.collection.fastingStatus.coding.code contains '{{C}}')",
        ),
        (
            "samplingDate",
            "exists from [Specimen] S\nwhere FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}}",
        ),
        (
            "fastingStatus",
            "exists from [Specimen] S\nwhere S.collection.fastingStatus.coding.code contains '{{C}}'",
        ),
        (
            "storageTemperature",
            "exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code '{{C}}' from {{A1}})",
        ),
    ]
    .into();

    map
});


static CRITERION_MAP: Lazy<HashMap<&str, Option<HashMap<&str, Vec<&str>>>>> = Lazy::new(|| {
    let map = [
        ("gender", Some([("type", vec!["gender"])].into())),
        (
            "diagnosis",
            Some(
                [
                    ("type", vec!["conditionSampleDiagnosis"]),
                    ("alias", vec!["icd10", "icd10gm"]),
                ]
                .into(),
            ),
        ),
        (
            "29463-7",
            Some(
                [
                    ("type", vec!["observationBodyWeight"]),
                    ("alias", vec!["loinc"]),
                ]
                .into(),
            ),
        ),
        (
            "39156-5",
            Some([("type", vec!["observationBMI"]), ("alias", vec!["loinc"])].into()),
        ),
        (
            "72166-2",
            Some([("type", vec!["observation"]), ("alias", vec!["loinc"])].into()),
        ),
        ("donor_age", Some([("type", vec!["age"])].into())),
        (
            "date_of_diagnosis",
            Some([("type", vec!["conditionRangeDate"])].into()),
        ),
        (
            "sample_kind",
            Some(
                [
                    ("type", vec!["specimen"]),
                    ("alias", vec!["SampleMaterialType"]),
                ]
                .into(),
            ),
        ),
        (
            "storage_temperature",
            Some(
                [
                    ("type", vec!["storageTemperature"]),
                    ("alias", vec!["StorageTemperature"]),
                ]
                .into(),
            ),
        ),
        (
            "pat_with_samples",
            Some([("type", vec!["hasSpecimen"])].into()),
        ),
        (
            "diagnosis_age_donor",
            Some([("type", vec!["conditionRangeAge"])].into()),
        ),
        (
            "fasting_status",
            Some(
                [
                    ("type", vec!["fastingStatus"]),
                    ("alias", vec!["FastingStatus"]),
                ]
                .into(),
            ),
        ),
        (
            "sampling_date",
            Some([("type", vec!["samplingDate"])].into()),
        ),
    ]
    .into();

    map
});



pub fn bbmri(ast: ast::Ast) -> String {

    let mut query: String = "(".to_string();

    let mut filter: String = "(".to_string();

    let mut code_systems: HashSet<String> = HashSet::new();

    let mut lists: String = "".to_string();

    match ast.ast.operand {
        ast::Operand::And => {
            query += " and ";

        },
        ast::Operand::Or => {
            query += " or ";
        },
    }

    for grandchild in ast.ast.children {

        process(grandchild, &mut query, &mut filter, &mut code_systems);

    }
    
    query += ")";

    
    
    for code_system in code_systems {
        lists = lists + format!("codesystem {}: '{}'", code_system.as_str(), ALIAS_BBMRI.get(code_system.as_str()).unwrap_or(&(""))).as_str();
    } 


    lists + query.as_str() + filter.as_str()
    
}

pub fn process(child: ast::Child, query: &mut String, filter: &mut String, code_systems: &mut HashSet<String> ) {

    let mut query_cond: String = "(".to_string();
    let mut filter_cond: String = "(".to_string();

    match child {
       
        ast::Child::Condition(condition) => {

            let condition_key_trans = condition.key.as_str();

            query_cond += condition_key_trans;

            match condition_key_trans {

                _ => {}
            }

            filter_cond += condition.key.as_str();


            let condition_type_trans = match condition.type_ {
                ast::ConditionType::Between => "between",
                ast::ConditionType::In => "in",
                ast::ConditionType::Equals => "equals",
                ast::ConditionType::NotEquals => "not_equals",
                ast::ConditionType::Contains => "contains",
                ast::ConditionType::GreaterThan => "greater than",
                ast::ConditionType::LowerThan => "lower than"
            };

            query_cond = query_cond + " " + condition_type_trans + " ";

            match condition.value {
                ast::ConditionValue::Boolean(value) => {
                    query_cond += value.to_string().as_str();
                },
                ast::ConditionValue::DateRange(date_range) => {
                    query_cond += date_range.min.as_str();
                    query_cond += ", ";
                    query_cond += date_range.max.as_str();
                },
                ast::ConditionValue::NumRange(num_range) => {
                    query_cond += num_range.min.to_string().as_str();
                    query_cond += ", ";
                    query_cond += num_range.max.to_string().as_str();
                },
                ast::ConditionValue::Number(value) => {
                    query_cond += value.to_string().as_str();
                },
                ast::ConditionValue::String(value) => {
                    query_cond += value.as_str();
                },
                ast::ConditionValue::StringArray(string_array) => {
                    for value in &string_array {
                        query_cond += value;
                        query_cond += ", ";
                    }
                    
                }

            } 

            
            query_cond += " ";
        },


        ast::Child::Operation(operation) => {
            match operation.operand {
                ast::Operand::And => {
                    query_cond += " and ";

                },
                ast::Operand::Or => {
                    query_cond += " or ";
                },
            }

            for grandchild in operation.children {
                process(grandchild, &mut query_cond, &mut filter_cond, code_systems);

            }

        },
    
    }
    
    query_cond += ")";
    filter_cond += ")";

    *query += query_cond.as_str();
    *filter += filter_cond.as_str();

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

    const ALL_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["male","other"]},{"children":[{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C25"},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C56"}],"de":"Diagnose ICD-10","en":"Diagnosis ICD-10","key":"diagnosis","operand":"OR"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-09-30T22:00:00.000Z"}},{"key":"BMI","system":"","type":"BETWEEN","value":{"max":100,"min":10}},{"key":"Body weight","system":"","type":"BETWEEN","value":{"max":1100,"min":10}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Other fasting status"]},{"key":"72166-2","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":10000,"min":100}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","blood-plasma","buffy-coat"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-29T23:00:00.000Z","min":"2023-10-03T22:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature-18to-35","temperature-60to-85"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const SOME_GBN: &str = r#"{"ast":{"children":[{"key":"gender","system":"","type":"IN","value":["other","male"]},{"key":"diagnosis","system":"http://fhir.de/CodeSystem/dimdi/icd-10-gm","type":"EQUALS","value":"C24"},{"key":"diagnosis_age_donor","system":"","type":"BETWEEN","value":{"max":11,"min":1}},{"key":"date_of_diagnosis","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"bmi","system":"","type":"BETWEEN","value":{"max":111,"min":1}},{"key":"body_weight","system":"","type":"BETWEEN","value":{"max":1111,"min":110}},{"key":"fasting_status","system":"","type":"IN","value":["Sober","Not sober"]},{"key":"smoking_status","system":"","type":"IN","value":["Smoker","Never smoked"]},{"key":"donor_age","system":"","type":"BETWEEN","value":{"max":123,"min":1}},{"key":"sample_kind","system":"","type":"IN","value":["blood-serum","tissue-other"]},{"key":"sampling_date","system":"","type":"BETWEEN","value":{"max":"2023-10-30T23:00:00.000Z","min":"2023-10-29T23:00:00.000Z"}},{"key":"storage_temperature","system":"","type":"IN","value":["temperature2to10","temperatureGN"]}],"de":"haupt","en":"main","key":"main","operand":"AND"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;
    
    const LENS2: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"},{"key":"gender","system":"","type":"EQUALS","value":"female"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"tissue-frozen"},{"key":"sample_kind","system":"","type":"EQUALS","value":"blood-serum"}],"operand":"OR"}],"operand":"AND"},{"children":[{"children":[{"key":"gender","system":"","type":"EQUALS","value":"male"}],"operand":"OR"},{"children":[{"key":"diagnosis","system":"","type":"EQUALS","value":"C41"},{"key":"diagnosis","system":"","type":"EQUALS","value":"C50"}],"operand":"OR"},{"children":[{"key":"sample_kind","system":"","type":"EQUALS","value":"liquid-other"},{"key":"sample_kind","system":"","type":"EQUALS","value":"rna"},{"key":"sample_kind","system":"","type":"EQUALS","value":"urine"}],"operand":"OR"},{"children":[{"key":"storage_temperature","system":"","type":"EQUALS","value":"temperatureRoom"},{"key":"storage_temperature","system":"","type":"EQUALS","value":"four_degrees"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#; 

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

        //println!("{:?}", CRITERION_MAP.get("gender"));

        //println!("{:?}",CRITERION_MAP);
    }
}
