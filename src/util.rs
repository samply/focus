use rand::distributions::Distribution;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statrs::distribution::Laplace;
use std::collections::HashMap;
use tracing::{debug, info, warn};

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
}

// #[derive(Debug, Deserialize, Serialize)]
// struct Stratum {
//     population: Vec<Population>,
//     value: HashMap<String, String>,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct Stratifier {
//     code: Vec<Code>,
//     stratum: Vec<Stratum>,
// }

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
    measure: String,
    period: Period,
    resource_type: String,
    status: String,
    type_: String, //because "type" is a reserved keyword
}

type Sensitivity = usize;
type Count = u64;
type Bin = usize;

pub struct ObfCache {
    pub cache: HashMap<(Sensitivity, Count, Bin), u64>,
}

const DELTA_PATIENT: f64 = 1.;
const DELTA_SPECIMEN: f64 = 20.;
const DELTA_DIAGNOSIS: f64 = 3.;
const EPSILON: f64 = 0.1;


pub(crate) fn get_json_field(json_string: &str, field: &str) -> Result<Value, serde_json::Error> {
    let json: Value = serde_json::from_str(json_string)?;
    Ok(json[field].clone())
}

pub(crate) fn replace_cql(decoded_library: impl Into<String>) -> String {
    let replace_map: HashMap<&str, &str> =
        [
            ("BORSCHTSCH_GENDER_STRATIFIER", 
            "define Gender:
             if (Patient.gender is null) then 'unknown' else Patient.gender"
            ),
            ("BORSCHTSCH_SAMPLE_TYPE_STRATIFIER", "define function SampleType(specimen FHIR.Specimen):\n    case FHIRHelpers.ToCode(specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').first())\n        when Code 'plasma-edta' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-citrat' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-heparin' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-cell-free' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-other' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-formalin' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'normal-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'other-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'normal-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'other-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-paxgene-or-else' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid' from SampleMaterialType then 'liquid-other' \n        when Code 'tissue' from SampleMaterialType then 'tissue-other' \n        when Code 'serum' from SampleMaterialType then 'blood-serum' \n        when Code 'cf-dna' from SampleMaterialType then 'dna' \n        when Code 'g-dna' from SampleMaterialType then 'dna' \n        when Code 'blood-plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-other' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative-other' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid-other' from SampleMaterialType then 'liquid-other' \n        when Code 'blood-serum' from SampleMaterialType then 'blood-serum' \n        when Code 'dna' from SampleMaterialType then 'dna' \n        when Code 'buffy-coat' from SampleMaterialType then 'buffy-coat' \n        when Code 'urine' from SampleMaterialType then 'urine' \n        when Code 'ascites' from SampleMaterialType then 'ascites' \n        when Code 'saliva' from SampleMaterialType then 'saliva' \n        when Code 'csf-liquor' from SampleMaterialType then 'csf-liquor' \n        when Code 'bone-marrow' from SampleMaterialType then 'bone-marrow' \n        when Code 'peripheral-blood-cells-vital' from SampleMaterialType then 'peripheral-blood-cells-vital' \n        when Code 'stool-faeces' from SampleMaterialType then 'stool-faeces' \n        when Code 'rna' from SampleMaterialType then 'rna' \n        when Code 'whole-blood' from SampleMaterialType then 'whole-blood' \n        when Code 'swab' from SampleMaterialType then 'swab' \n        when Code 'dried-whole-blood' from SampleMaterialType then 'dried-whole-blood' \n        when null  then 'Unknown'\n        else 'Unknown'\n    end"),
            ("BORSCHTSCH_CUSTODIAN_STRATIFIER", "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)"),
            ("BORSCHTSCH_DIAGNOSIS_STRATIFIER", "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n"),
            ("BORSCHTSCH_AGE_STRATIFIER", "define AgeClass:\n     (AgeInYears() div 10) * 10"),
            ("BORSCHTSCH_DEF_SPECIMEN", "define Specimen:"),
            ("BORSCHTSCH_DEF_IN_INITIAL_POPULATION", "define InInitialPopulation:")
        ].into();

    let mut decoded_library = decoded_library.into();

    for (key, value) in replace_map {
        let replacement_value = value.to_string() + "\n";
        decoded_library = decoded_library.replace(key, &replacement_value[..]);
    }

    decoded_library
}

pub(crate) fn is_cql_tampered_with(decoded_library: impl Into<String>) -> bool {
    let decoded_library = decoded_library.into();
    decoded_library.contains("define")
}

pub(crate) fn obfuscate_counts(json_str: &str, obf_cache: &mut ObfCache) -> String {

    let patient_dist = Laplace::new(0.0, DELTA_PATIENT/EPSILON).unwrap(); //TODO error handling
    let diagnosis_dist = Laplace::new(0.0, DELTA_DIAGNOSIS/EPSILON).unwrap();
    let specimen_dist = Laplace::new(0.0, DELTA_SPECIMEN/EPSILON).unwrap();

    let mut measure_report: MeasureReport = serde_json::from_str(&json_str).unwrap();
    for g in &mut measure_report.group {
        match &g.code.text[..] {
            "patients" => {
                info!("patients");
                obfuscate_counts_recursive(&mut g.population, &patient_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &patient_dist, 2, obf_cache);
            }
            "diagnosis" => {
                info!("diagnosis");
                obfuscate_counts_recursive(&mut g.population, &diagnosis_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &diagnosis_dist, 2, obf_cache);
            }
            "specimen" => {
                info!("specimen");
                obfuscate_counts_recursive(&mut g.population, &specimen_dist, 1, obf_cache);
                obfuscate_counts_recursive(&mut g.stratifier, &specimen_dist, 2, obf_cache);
            }
            _ => {
                warn!("focus is not aware of this type of stratifier")
            }
        }
    }

    let measure_report_obfuscated = serde_json::to_string_pretty(&measure_report).unwrap(); //TODO error handling
    dbg!(measure_report_obfuscated.clone());
    measure_report_obfuscated
}

fn obfuscate_counts_recursive(val: &mut Value, dist: &Laplace, bin: Bin, obf_cache: &mut ObfCache) {
    match val {
        Value::Object(map) => {
            if let Some(count_val) = map.get_mut("count") {
                if let Some(count) = count_val.as_u64() {
                    if count >= 1 && count <= 10 {
                        *count_val = json!(10);
                    } else if count > 10 {
                        let mut rng = thread_rng();
                        let sensitivity: usize = (dist.scale() * EPSILON).round() as usize;
                        
                        let pertubation = match obf_cache.cache.get(&(sensitivity, count, bin)) {
                            Some(pertubation_reference) => *pertubation_reference,
                            None => {
                                let pertubation_value = dist.sample(&mut rng).round() as u64;
                                obf_cache.cache.insert((sensitivity, count, bin), pertubation_value);
                                pertubation_value
                            }
                        };

                        *count_val = json!((count + pertubation + 5) / 10 * 10);
                        // Per data protection concept it must be rounded to the nearest multiple of 10
                        // "Counts of patients and samples undergo obfuscation on site before being sent to central infrastructure. This is done by incorporating some randomness into the count and then rounding it to the nearest multiple of ten."
                    } // And zero stays zero
                }
            }
            for (_, sub_val) in map.iter_mut() {
                obfuscate_counts_recursive(sub_val, dist, bin, obf_cache);
            }
        }
        Value::Array(vec) => {
            for sub_val in vec.iter_mut() {
                obfuscate_counts_recursive(sub_val, dist, bin, obf_cache);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

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
        assert_eq!(result.unwrap(), expected_result);
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
        assert_eq!(result.unwrap(), json!(null));
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
        let decoded_library = "context Patient\nBORSCHTSCH_GENDER_STRATIFIER";
        assert!(!is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_replace_cql() {
        let decoded_library = "BORSCHTSCH_GENDER_STRATIFIER";
        let expected_result = "define Gender:\n             if (Patient.gender is null) then 'unknown' else Patient.gender\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_CUSTODIAN_STRATIFIER";
        let expected_result = "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DIAGNOSIS_STRATIFIER";
        let expected_result = "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_AGE_STRATIFIER";
        let expected_result = "define AgeClass:\n     (AgeInYears() div 10) * 10\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DEF_SPECIMEN";
        let expected_result = "define Specimen:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DEF_IN_INITIAL_POPULATION";
        let expected_result = "define InInitialPopulation:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "INVALID_KEY";
        let expected_result = "INVALID_KEY";
        assert_eq!(replace_cql(decoded_library), expected_result);
    }
}
