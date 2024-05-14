use crate::errors::FocusError;
use base64::engine::general_purpose;
use base64::Engine as _;
use laplace_rs::{get_from_cache_or_privatize, Bin, ObfCache, ObfuscateBelow10Mode};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use tracing::warn;

#[derive(Debug, Deserialize, Serialize)]
struct Period {
    end: String,
    start: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ValueQuantity {
    code: String,
    system: String,
    unit: String,
    value: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Extension {
    url: String,
    value_quantity: ValueQuantity,
}

#[derive(Debug, Deserialize, Serialize)]
struct Code {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Coding {
    code: String,
    system: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Population {
    code: Code,
    count: u64,
    subject_results: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct Group {
    code: Code,
    population: Value,
    stratifier: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeasureReport {
    date: String,
    extension: Vec<Extension>,
    group: Vec<Group>,
    id: Option<String>,
    measure: String,
    meta: Option<Value>,
    period: Period,
    resource_type: String,
    status: String,
    type_: String, //because "type" is a reserved keyword
}

pub(crate) fn get_json_field(json_string: &str, field: &str) -> Result<Value, serde_json::Error> {
    let json: Value = serde_json::from_str(json_string)?;
    Ok(json[field].clone())
}

pub(crate) fn read_lines(filename: String) -> Result<io::Lines<BufReader<File>>, FocusError> {
    let file = File::open(filename.clone()).map_err(|e| {
        FocusError::FileOpeningError(format!("Cannot open file {}: {} ", filename, e))
    })?;
    Ok(io::BufReader::new(file).lines())
}

pub(crate) fn base64_decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, FocusError> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(FocusError::DecodeError)
}

// REPLACE_MAP is built in build.rs
include!(concat!(env!("OUT_DIR"), "/replace_map.rs"));

pub(crate) fn replace_cql(decoded_library: impl Into<String>) -> String {
    let mut decoded_library = decoded_library.into();

    for (key, value) in REPLACE_MAP.iter() {
        decoded_library = decoded_library.replace(key, &value[..]);
    }
    decoded_library
}

pub(crate) fn is_cql_tampered_with(decoded_library: impl Into<String>) -> bool {
    let decoded_library = decoded_library.into();
    decoded_library.contains("define")
}

pub fn obfuscate_counts_mr(
    json_str: &str,
    obf_cache: &mut ObfCache,
    obfuscate_zero: bool,
    obfuscate_below_10_mode: usize,
    delta_patient: f64,
    delta_specimen: f64,
    delta_diagnosis: f64,
    delta_procedures: f64,
    delta_medication_statements: f64,
    delta_histo: f64,
    epsilon: f64,
    rounding_step: usize,
) -> Result<String, FocusError> {
    let obf_10: ObfuscateBelow10Mode = match obfuscate_below_10_mode {
        0 => ObfuscateBelow10Mode::Zero,
        1 => ObfuscateBelow10Mode::Ten,
        2 => ObfuscateBelow10Mode::Obfuscate,
        _ => ObfuscateBelow10Mode::Obfuscate,
    };
    let mut measure_report: MeasureReport = serde_json::from_str(&json_str)
        .map_err(|e| FocusError::DeserializationError(format!(r#"{}. Is obfuscation turned on when it shouldn't be? Is the metadata in the task formatted correctly, like this {{"project": "name"}}? Are there any other projects stated in the projects_no_obfuscation parameter in the bridgehead?"#, e)))?;
    for g in &mut measure_report.group {
        match &g.code.text[..] {
            "patient" | "patients" => { // Prism used "patient" for catalogue, Lens uses "patients"
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_patient,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_patient,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            "diagnosis" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_diagnosis,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_diagnosis,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            "specimen" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_specimen,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_specimen,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            "procedures" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_procedures,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_procedures,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            "medicationStatements" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_medication_statements,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_medication_statements,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            "Histo" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    delta_histo,
                    epsilon,
                    1,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
                obfuscate_counts_recursive(
                    &mut g.stratifier,
                    delta_histo,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            }
            _ => {
                warn!("Focus is not aware of {} type of stratifier, therefore it will not obfuscate the values.", &g.code.text[..])
            }
        }
    }

    let measure_report_obfuscated = serde_json::to_string_pretty(&measure_report)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;
    Ok(measure_report_obfuscated)
}

fn obfuscate_counts_recursive(
    val: &mut Value,
    delta: f64,
    epsilon: f64,
    bin: Bin,
    obf_cache: &mut ObfCache,
    obfuscate_zero: bool,
    obfuscate_below_10_mode: ObfuscateBelow10Mode,
    rounding_step: usize,
) -> Result<(), FocusError> {
    let mut rng = thread_rng();
    match val {
        Value::Object(map) => {
            if let Some(count_val) = map.get_mut("count") {
                if let Some(count) = count_val.as_u64() {
                    if (1..=10).contains(&count) {
                        *count_val = json!(10);
                    } else {
                        let obfuscated = get_from_cache_or_privatize(
                            count,
                            delta,
                            epsilon,
                            bin,
                            Some(obf_cache),
                            obfuscate_zero,
                            obfuscate_below_10_mode.clone(),
                            rounding_step,
                            &mut rng,
                        )
                        .map_err(FocusError::LaplaceError);

                        *count_val = json!(obfuscated?);
                    }
                }
            }
            for (_, sub_val) in map.iter_mut() {
                obfuscate_counts_recursive(
                    sub_val,
                    delta,
                    epsilon,
                    bin,
                    obf_cache,
                    obfuscate_zero,
                    obfuscate_below_10_mode.clone(),
                    rounding_step,
                )?;
            }
        }
        Value::Array(vec) => {
            for sub_val in vec.iter_mut() {
                obfuscate_counts_recursive(
                    sub_val,
                    delta,
                    epsilon,
                    bin,
                    obf_cache,
                    obfuscate_zero,
                    obfuscate_below_10_mode.clone(),
                    rounding_step,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    const QUERY_BBMRI_PLACEHOLDERS: &str =
        include_str!("../resources/test/query_bbmri_placeholders.cql");
    const QUERY_BBMRI: &str = include_str!("../resources/test/query_bbmri.cql");
    const EXAMPLE_MEASURE_REPORT_BBMRI: &str =
        include_str!("../resources/test/measure_report_bbmri.json");
    const EXAMPLE_MEASURE_REPORT_DKTK: &str =
        include_str!("../resources/test/measure_report_dktk.json");
    const EXAMPLE_MEASURE_REPORT_EXLIQUID: &str =
        include_str!("../resources/test/measure_report_exliquid.json");

    const DELTA_PATIENT: f64 = 1.;
    const DELTA_SPECIMEN: f64 = 20.;
    const DELTA_DIAGNOSIS: f64 = 3.;
    const DELTA_PROCEDURES: f64 = 1.7;
    const DELTA_MEDICATION_STATEMENTS: f64 = 2.1;
    const DELTA_HISTO: f64 = 20.;
    const EPSILON: f64 = 0.1;
    const ROUNDING_STEP: usize = 10;

    #[test]
    fn test_get_json_field_success() {
        let json_string = r#"
            {
                "name": "FHIRy McFHIRFace",
                "age": 47,
                "address": {
                    "street": "Brückenkopfstrasse 1",
                    "city": "Heidelberg",
                    "state": "BW",
                    "zip": "69120"
                }
            }
        "#;
        let expected_result = serde_json::json!({
            "street": "Brückenkopfstrasse 1",
            "city": "Heidelberg",
            "state": "BW",
            "zip": "69120"
        });

        // Call the function and assert that it returns the expected result
        let result = get_json_field(json_string, "address");
        assert!(result.is_ok());
        pretty_assertions::assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn test_get_json_field_nonexistent_field() {
        let json_string = r#"
            {
                "name": "FHIRy McFHIRFace",
                "age": 47,
                "address": {
                    "street": "Brückenkopfstrasse 1",
                    "city": "Heidelberg",
                    "state": "BW",
                    "zip": "69120"
                }
            }
        "#;

        // Call the function and assert that it returns json null
        let result = get_json_field(json_string, "phone");
        assert!(result.is_ok());
        pretty_assertions::assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_get_json_field_invalid_json() {
        let json_string = r#"{"name": "FHIRy McFHIRFace", "age": 47"#;

        // Call the function and assert that it returns an error
        let result = get_json_field(json_string, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_cql_tampered_with_true() {
        let decoded_library =
            "define Gender:\n if (Patient.gender is null) then 'unknown' else Patient.gender \n";
        assert!(is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_is_cql_tampered_with_false() {
        let decoded_library = "context Patient\nBBMRI_STRAT_GENDER_STRATIFIER";
        assert!(!is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_replace_cql_all() {
        for (decoded_library, expected_result) in REPLACE_MAP.iter() {
            pretty_assertions::assert_eq!(replace_cql(*decoded_library).as_str(), *expected_result);
        }
    }

    #[test]
    fn test_replace_cql() {
        let decoded_library = QUERY_BBMRI_PLACEHOLDERS;
        let expected_result = QUERY_BBMRI;
        pretty_assertions::assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "INVALID_KEY";
        let expected_result = "INVALID_KEY";
        pretty_assertions::assert_eq!(replace_cql(decoded_library), expected_result);
    }

    #[test]
    fn test_obfuscate_counts_bbmri() {
        let mut obf_cache = ObfCache {
            cache: HashMap::new(),
        };
        let obfuscated_json = obfuscate_counts_mr(
            EXAMPLE_MEASURE_REPORT_BBMRI,
            &mut obf_cache,
            false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            DELTA_HISTO,
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();

        // Check that the obfuscated JSON can be parsed and has the same structure as the original JSON
        let _: MeasureReport = serde_json::from_str(&obfuscated_json).unwrap();

        // Check that the obfuscated JSON is different from the original JSON
        assert_ne!(obfuscated_json, EXAMPLE_MEASURE_REPORT_BBMRI);

        // Check that obfuscating the same JSON twice with the same obfuscation cache gives the same result
        let obfuscated_json_2 = obfuscate_counts_mr(
            EXAMPLE_MEASURE_REPORT_BBMRI,
            &mut obf_cache,
            false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            DELTA_HISTO,
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();
        pretty_assertions::assert_eq!(obfuscated_json, obfuscated_json_2);
    }

    #[test]
    fn test_obfuscate_counts_dktk() {
        let mut obf_cache = ObfCache {
            cache: HashMap::new(),
        };
        let obfuscated_json = obfuscate_counts_mr(
            EXAMPLE_MEASURE_REPORT_DKTK,
            &mut obf_cache,
            false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            DELTA_HISTO,
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();

        // Check that the obfuscated JSON can be parsed and has the same structure as the original JSON
        let _: MeasureReport = serde_json::from_str(&obfuscated_json).unwrap();

        // Check that the obfuscated JSON is different from the original JSON
        pretty_assertions::assert_ne!(obfuscated_json, EXAMPLE_MEASURE_REPORT_DKTK);

        // Check that obfuscating the same JSON twice with the same obfuscation cache gives the same result
        let obfuscated_json_2 = obfuscate_counts_mr(
            EXAMPLE_MEASURE_REPORT_DKTK,
            &mut obf_cache,
            false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            DELTA_HISTO,
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();
        pretty_assertions::assert_eq!(obfuscated_json, obfuscated_json_2);
    }

    #[test]
    fn test_obfuscate_counts_bad_measure() {
        let mut obf_cache = ObfCache {
            cache: HashMap::new(),
        };
        let obfuscated_json = obfuscate_counts_mr(
            EXAMPLE_MEASURE_REPORT_EXLIQUID,
            &mut obf_cache,
            false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            DELTA_HISTO,
            EPSILON,
            ROUNDING_STEP,
        );

        pretty_assertions::assert_eq!(
            obfuscated_json.unwrap_err().to_string(),
            r#"Deserialization error: missing field `text` at line 42 column 13. Is obfuscation turned on when it shouldn't be? Is the metadata in the task formatted correctly, like this {"project": "name"}? Are there any other projects stated in the projects_no_obfuscation parameter in the bridgehead?"#
        );
    }
}
