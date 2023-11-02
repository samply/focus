use crate::errors::FocusError;
use std::fs::File;
use std::io::{ self, BufRead, BufReader };
use laplace_rs::{get_from_cache_or_privatize, Bin, ObfCache, ObfuscateBelow10Mode};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
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

const MU: f64 = 0.;

pub(crate) fn get_json_field(json_string: &str, field: &str) -> Result<Value, serde_json::Error> {
    let json: Value = serde_json::from_str(json_string)?;
    Ok(json[field].clone())
}

pub(crate) fn read_lines(filename: String) -> Result<io::Lines<BufReader<File>>, FocusError> {
    let file = File::open(filename.clone()).map_err(|e| {
        FocusError::FileOpeningError(format!(
            "Cannot open file {}: {} ", 
            filename,
            e
        ))
    })?; 
    Ok(io::BufReader::new(file).lines())
}

pub(crate) fn replace_cql(decoded_library: impl Into<String>) -> String {
    let replace_map: HashMap<&str, &str> =
        [
            ("BBMRI_STRAT_GENDER_STRATIFIER", 
            "define Gender:
             if (Patient.gender is null) then 'unknown' else Patient.gender"
            ),
            ("BBMRI_STRAT_SAMPLE_TYPE_STRATIFIER", "define function SampleType(specimen FHIR.Specimen):\n    case FHIRHelpers.ToCode(specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').first())\n        when Code 'plasma-edta' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-citrat' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-heparin' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-cell-free' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-other' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-formalin' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'normal-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'other-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'normal-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'other-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-paxgene-or-else' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid' from SampleMaterialType then 'liquid-other' \n        when Code 'tissue' from SampleMaterialType then 'tissue-other' \n        when Code 'serum' from SampleMaterialType then 'blood-serum' \n        when Code 'cf-dna' from SampleMaterialType then 'dna' \n        when Code 'g-dna' from SampleMaterialType then 'dna' \n        when Code 'blood-plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-other' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative-other' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid-other' from SampleMaterialType then 'liquid-other' \n        when Code 'blood-serum' from SampleMaterialType then 'blood-serum' \n        when Code 'dna' from SampleMaterialType then 'dna' \n        when Code 'buffy-coat' from SampleMaterialType then 'buffy-coat' \n        when Code 'urine' from SampleMaterialType then 'urine' \n        when Code 'ascites' from SampleMaterialType then 'ascites' \n        when Code 'saliva' from SampleMaterialType then 'saliva' \n        when Code 'csf-liquor' from SampleMaterialType then 'csf-liquor' \n        when Code 'bone-marrow' from SampleMaterialType then 'bone-marrow' \n        when Code 'peripheral-blood-cells-vital' from SampleMaterialType then 'peripheral-blood-cells-vital' \n        when Code 'stool-faeces' from SampleMaterialType then 'stool-faeces' \n        when Code 'rna' from SampleMaterialType then 'rna' \n        when Code 'whole-blood' from SampleMaterialType then 'whole-blood' \n        when Code 'swab' from SampleMaterialType then 'swab' \n        when Code 'dried-whole-blood' from SampleMaterialType then 'dried-whole-blood' \n        when null  then 'Unknown'\n        else 'Unknown'\n    end"),
            ("BBMRI_STRAT_CUSTODIAN_STRATIFIER", "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)"),
            ("BBMRI_STRAT_DIAGNOSIS_STRATIFIER", "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n"),
            ("BBMRI_STRAT_AGE_STRATIFIER", "define AgeClass:\n     (AgeInYears() div 10) * 10"),
            ("BBMRI_STRAT_DEF_SPECIMEN", "define Specimen:"),
            ("BBMRI_STRAT_DEF_IN_INITIAL_POPULATION", "define InInitialPopulation:"),
            ("EXLIQUID_CQL", "define ExliquidSpecimen:\nif InInitialPopulation then [Specimen] S\n where S.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen'\ndefine retrieveCondition:\n First(from [Condition] C\n  return C.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\ndefine Diagnosis:\n  if (retrieveCondition is null) then 'unknown' else retrieveCondition\ndefine function SampleType(specimen FHIR.Specimen):\n  specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()"),
            ("EXLIQUID_ALIQUOTS_CQL", "define Aliquot:\nif InInitialPopulation then [Specimen] S\n where exists S.collection.quantity.value and exists S.parent.reference and S.container.specimenQuantity.value > 0 define AliquotGroupReferences: flatten Aliquot S return S.parent.reference define AliquotGroupWithAliquot: [Specimen] S where not (S.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen') and not exists S.collection.quantity.value and not exists S.container.specimenQuantity.value and AliquotGroupReferences contains 'Specimen/' + S.id define PrimarySampleReferences: flatten AliquotGroupWithAliquot S return S.parent.reference define ExliquidSpecimenWithAliquot: from [Specimen] PrimarySample where PrimarySample.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen'   and PrimarySampleReferences contains 'Specimen/' + PrimarySample.id    define retrieveCondition:   First(from [Condition] C   return C.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())    define Diagnosis:   if (retrieveCondition is null) then 'unknown' else retrieveCondition define function SampleType(specimen FHIR.Specimen): specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()"),
            ("DKTK_STRAT_GENDER_STRATIFIER", "define Gender:\nif (Patient.gender is null) then 'unknown' else Patient.gender"),
            ("DKTK_STRAT_AGE_STRATIFIER", "define PrimaryDiagnosis:\nFirst(\nfrom [Condition] C\nwhere C.extension.where(url='http://hl7.org/fhir/StructureDefinition/condition-related').empty()\nsort by date from onset asc)\n\ndefine AgeClass:\nif (PrimaryDiagnosis.onset is null) then 'unknown' else ToString((AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) div 10) * 10)"),
            ("DKTK_STRAT_DECEASED_STRATIFIER", "define PatientDeceased:\nFirst (from [Observation: Code '75186-7' from loinc] O return O.value.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VitalstatusCS').code.first())\ndefine Deceased:\nif (PatientDeceased is null) then 'unbekannt' else PatientDeceased"),
            ("DKTK_STRAT_DIAGNOSIS_STRATIFIER", "define Diagnosis:\nif InInitialPopulation then [Condition] else {} as List<Condition>\n\ndefine function DiagnosisCode(condition FHIR.Condition):\ncondition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first()"),
            ("DKTK_STRAT_SPECIMEN_STRATIFIER", "define Specimen:\nif InInitialPopulation then [Specimen] else {} as List<Specimen>\n\ndefine function SampleType(specimen FHIR.Specimen):\nspecimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()"),
            ("UCT_STRAT_SPECIMEN_STRATIFIER", "define Specimen:\nif InInitialPopulation then [Specimen] else {} as List<Specimen>\n\ndefine function SampleType(specimen FHIR.Specimen):\nspecimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()\n\ndefine function Lagerort(specimen FHIR.Specimen):\nspecimen.extension.where(url = 'http://uct-locator/specimen/storage').value.coding.code.first()\n\ndefine function annotations(specimen FHIR.Specimen):\n(if (specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first() is null) then 1 else 0) +\n(if (specimen.collection.collected is null) then 1 else 0)"),
            ("DKTK_STRAT_PROCEDURE_STRATIFIER", "define Procedure:\nif InInitialPopulation then [Procedure] else {} as List <Procedure>\n\ndefine function ProcedureType(procedure FHIR.Procedure):\nprocedure.category.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SYSTTherapieartCS').code.first()"),
            ("DKTK_STRAT_MEDICATION_STRATIFIER", "define MedicationStatement:\nif InInitialPopulation then [MedicationStatement] else {} as List <MedicationStatement>"),
            ("DKTK_STRAT_ENCOUNTER_STRATIFIER", "define Encounter:\nif InInitialPopulation then [Encounter] else {} as List<Encounter>\n\ndefine function Departments(encounter FHIR.Encounter):\nencounter.identifier.where(system = 'http://dktk.dkfz.de/fhir/sid/hki-department').value.first()"),
            ("DKTK_STRAT_DEF_IN_INITIAL_POPULATION", "define InInitialPopulation:"),
            ("EXLIQUID_STRAT_DEF_IN_INITIAL_POPULATION", "define InInitialPopulation:\n   exists ExliquidSpecimen and \n"),        
            ("EXLIQUID_STRAT_W_ALIQUOTS", "define InInitialPopulation: exists ExliquidSpecimenWithAliquot and \n")
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
        .map_err(|e| FocusError::DeserializationError(e.to_string()))?;
    for g in &mut measure_report.group {
        match &g.code.text[..] {
            "patients" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    MU,
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
                    MU,
                    delta_patient,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            },
            "diagnosis" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    MU,
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
                    MU,
                    delta_diagnosis,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            },
            "specimen" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    MU,
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
                    MU,
                    delta_specimen,
                    epsilon,
                    2,
                    obf_cache,
                    obfuscate_zero,
                    obf_10.clone(),
                    rounding_step,
                )?;
            },
                "procedures" => {
                    obfuscate_counts_recursive(
                        &mut g.population,
                        MU,
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
                        MU,
                        delta_procedures,
                        epsilon,
                        2,
                        obf_cache,
                        obfuscate_zero,
                        obf_10.clone(),
                        rounding_step,
                    )?;
            },
            "medicationStatements" => {
                obfuscate_counts_recursive(
                    &mut g.population,
                    MU,
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
                    MU,
                    delta_medication_statements,
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
    mu: f64,
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
                    if count >= 1 && count <= 10 {
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
                        .map_err(|e| FocusError::LaplaceError(e));

                        *count_val = json!(obfuscated?);
                    }
                }
            }
            for (_, sub_val) in map.iter_mut() {
                obfuscate_counts_recursive(
                    sub_val,
                    mu,
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
                    mu,
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

    const EXAMPLE_MEASURE_REPORT_BBMRI: &str = r#"{"date": "2023-03-03T15:54:21.740195064Z", "extension": [{"url": "https://samply.github.io/blaze/fhir/StructureDefinition/eval-duration", "valueQuantity": {"code": "s", "system": "http://unitsofmeasure.org", "unit": "s", "value": 0.050495759}}], "group": [{"code": {"text": "patients"}, "population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 74}], "stratifier": [{"code": [{"text": "Gender"}], "stratum": [{"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 31}], "value": {"text": "female"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 43}], "value": {"text": "male"}}]}, {"code": [{"text": "Age"}], "stratum": [{"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 5}], "value": {"text": "40"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 4}], "value": {"text": "50"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 14}], "value": {"text": "60"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 4}], "value": {"text": "80"}}]}, {"code": [{"text": "Custodian"}], "stratum": [{"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 31}], "value": {"text": "bbmri-eric:ID:CZ_CUNI_PILS:collection:serum_plasma"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 43}], "value": {"text": "null"}}]}]}, {"code": {"text": "diagnosis"}, "population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 324}], "stratifier": [{"code": [{"text": "diagnosis"}], "stratum": [{"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 26}], "value": {"text": "C34.0"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 28}], "value": {"text": "C34.2"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 25}], "value": {"text": "C34.8"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 27}], "value": {"text": "C78.0"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 25}], "value": {"text": "D38.6"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 25}], "value": {"text": "R91"}}]}]}, {"code": {"text": "specimen"}, "population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 124}], "stratifier": [{"code": [{"text": "sample_kind"}], "stratum": [{"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 62}], "value": {"text": "blood-plasma"}}, {"population": [{"code": {"coding": [{"code": "initial-population", "system": "http://terminology.hl7.org/CodeSystem/measure-population"}]}, "count": 62}], "value": {"text": "blood-serum"}}]}]}], "measure": "urn:uuid:fe7e5bf7-d792-4368-b1d2-5798930db13e", "period": {"end": "2030", "start": "2000"}, "resourceType": "MeasureReport", "status": "complete", "type": "summary", "meta": {"lastUpdated": "2023-06-27T13:12:56.719Z", "versionId": "11"}, "id": "DCH47RNHPKH6TC3W"}"#;
    const EXAMPLE_MEASURE_REPORT_DKTK: &str = r#"{"date":"2023-10-17T13:54:31.879752655Z","extension":[{"url":"https://samply.github.io/blaze/fhir/StructureDefinition/eval-duration","valueQuantity":{"code":"s","system":"http://unitsofmeasure.org","unit":"s","value":0.227357643}}],"group":[{"code":{"text":"patients"},"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4228}],"stratifier":[{"code":[{"text":"Gender"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4228}],"value":{"text":"male"}}]},{"code":[{"text":"75186-7"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3460}],"value":{"text":"lebend"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":768}],"value":{"text":"verstorben"}}]},{"code":[{"text":"Age"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":20}],"value":{"text":"0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":40}],"value":{"text":"10"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"100"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":89}],"value":{"text":"20"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":165}],"value":{"text":"30"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":227}],"value":{"text":"40"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":644}],"value":{"text":"50"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1027}],"value":{"text":"60"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1150}],"value":{"text":"70"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":769}],"value":{"text":"80"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":91}],"value":{"text":"90"}}]}]},{"code":{"text":"diagnosis"},"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4461}],"stratifier":[{"code":[{"text":"diagnosis"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":25}],"value":{"text":"C01"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C02.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":23}],"value":{"text":"C02.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C02.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C02.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C03.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":20}],"value":{"text":"C03.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C04.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"C04.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C04.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C04.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C05.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C05.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C05.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C06.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C06.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C06.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C06.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C07"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C08.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C08.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C09.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C09.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C09.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":17}],"value":{"text":"C09.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C10.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C10.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C10.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C10.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C11"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C11.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C11.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C11.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C11.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C12"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C13"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C13.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C13.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C13.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C14.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C15.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C15.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C15.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C15.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":15}],"value":{"text":"C15.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":49}],"value":{"text":"C15.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C15.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C15.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":38}],"value":{"text":"C16.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C16.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C16.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C16.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C16.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C16.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C17.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C17.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C17.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C18"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C18.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C18.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":17}],"value":{"text":"C18.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C18.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C18.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C18.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C18.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":32}],"value":{"text":"C18.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C18.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C19"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":97}],"value":{"text":"C20"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C21.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C21.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C21.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":39}],"value":{"text":"C22.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":12}],"value":{"text":"C22.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C22.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C22.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C23"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C24.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C24.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C25"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":26}],"value":{"text":"C25.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":14}],"value":{"text":"C25.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C25.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C25.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C25.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C25.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C25.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C26.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":20}],"value":{"text":"C30.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C31"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C31.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C31.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C31.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C31.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":30}],"value":{"text":"C32.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C32.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C32.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C32.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C32.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C34"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C34.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":144}],"value":{"text":"C34.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"C34.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":71}],"value":{"text":"C34.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":33}],"value":{"text":"C34.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":35}],"value":{"text":"C34.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C37"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C38.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C38.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C38.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C40.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C40.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C40.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C40.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C41.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C41.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C41.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C41.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C43.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C43.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C43.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":50}],"value":{"text":"C43.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":30}],"value":{"text":"C43.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":25}],"value":{"text":"C43.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C43.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C44.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":12}],"value":{"text":"C44.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":37}],"value":{"text":"C44.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":104}],"value":{"text":"C44.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":44}],"value":{"text":"C44.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":29}],"value":{"text":"C44.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":28}],"value":{"text":"C44.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":25}],"value":{"text":"C44.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C44.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C45.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C45.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C46.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C47.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C47.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C47.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":14}],"value":{"text":"C48.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C49.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":18}],"value":{"text":"C49.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":40}],"value":{"text":"C49.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C49.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"C49.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":12}],"value":{"text":"C49.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C49.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C49.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C50.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C50.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C50.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C50.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C60.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C60.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C60.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":541}],"value":{"text":"C61"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C62.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C62.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C63.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C63.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C63.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":54}],"value":{"text":"C64"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C65"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C66"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C67.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C67.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":23}],"value":{"text":"C67.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":25}],"value":{"text":"C67.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C68.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C68.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C69.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C69.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C69.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C69.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C70.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C71.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":35}],"value":{"text":"C71.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":46}],"value":{"text":"C71.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":16}],"value":{"text":"C71.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C71.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C71.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C71.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C71.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C71.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":23}],"value":{"text":"C71.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C72.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C72.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":121}],"value":{"text":"C73"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C74.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C75.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C76.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":55}],"value":{"text":"C80.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C81.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":17}],"value":{"text":"C81.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"C81.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C81.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C81.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C82.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":17}],"value":{"text":"C82.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C82.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C82.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"C83.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C83.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":86}],"value":{"text":"C83.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C83.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C83.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":11}],"value":{"text":"C84.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C84.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"C84.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C84.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C84.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"C85.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C85.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C85.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C86.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C86.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C86.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C86.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C86.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C86.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C88.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"C88.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":107}],"value":{"text":"C90.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":42}],"value":{"text":"C91.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":15}],"value":{"text":"C91.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C91.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C91.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C91.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C91.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":69}],"value":{"text":"C92.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"C92.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C92.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"C92.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":23}],"value":{"text":"C92.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":29}],"value":{"text":"C92.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"C93.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":15}],"value":{"text":"C93.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C94.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"C94.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C95.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C96.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"C96.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"C96.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"C96.8"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5}],"value":{"text":"D00.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D00.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D01.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D01.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D01.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D02.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D02.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D03.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":10}],"value":{"text":"D03.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D03.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":6}],"value":{"text":"D03.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D03.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"D03.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D04.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D07.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D07.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"D07.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":24}],"value":{"text":"D09.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9}],"value":{"text":"D09.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":20}],"value":{"text":"D32.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D32.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D33.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D33.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D33.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"D33.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":75}],"value":{"text":"D35.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D37.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D37.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D37.7"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D38.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D38.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D42.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D43.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"D43.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"D43.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D43.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D44.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":1}],"value":{"text":"D44.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D45"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":29}],"value":{"text":"D46.2"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":20}],"value":{"text":"D46.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D46.6"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":8}],"value":{"text":"D46.9"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D47.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D47.3"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":25}],"value":{"text":"D47.4"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3}],"value":{"text":"D48.0"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":7}],"value":{"text":"D48.1"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2}],"value":{"text":"D48.5"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":664}],"value":{"text":"null"}}]}]},{"code":{"text":"specimen"},"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":627}],"stratifier":[{"code":[{"text":"sample_kind"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":114}],"value":{"text":"blood-serum"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":154}],"value":{"text":"bone-marrow"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":154}],"value":{"text":"liquid-other"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"tissue-other"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":201}],"value":{"text":"whole-blood"}}]}]},{"code":{"text":"procedures"},"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":9377}],"stratifier":[{"code":[{"text":"ProcedureType"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":3770}],"value":{"text":"OP"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":5607}],"value":{"text":"ST"}}]}]},{"code":{"text":"medicationStatements"},"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4236}],"stratifier":[{"code":[{"text":"MedicationType"}],"stratum":[{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":2840}],"value":{"text":"CH"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":75}],"value":{"text":"HO"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":517}],"value":{"text":"IM"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":496}],"value":{"text":"KM"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":274}],"value":{"text":"SO"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":17}],"value":{"text":"WS"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":13}],"value":{"text":"ZS"}},{"population":[{"code":{"coding":[{"code":"initial-population","system":"http://terminology.hl7.org/CodeSystem/measure-population"}]},"count":4}],"value":{"text":"null"}}]}]}],"measure":"urn:uuid:d339032e-0d7e-4610-99e8-dad80bc2fca1","period":{"end":"2030","start":"2000"},"resourceType":"MeasureReport","status":"complete","type":"summary"}"#;

    const DELTA_PATIENT: f64 = 1.;
    const DELTA_SPECIMEN: f64 = 20.;
    const DELTA_DIAGNOSIS: f64 = 3.;
    const DELTA_PROCEDURES: f64 = 1.7;
    const DELTA_MEDICATION_STATEMENTS: f64 = 2.1;
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
        let decoded_library = "context Patient\nBBMRI_STRAT_GENDER_STRATIFIER";
        assert!(!is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_replace_cql() {
        let decoded_library = "BBMRI_STRAT_GENDER_STRATIFIER";
        let expected_result = "define Gender:\n             if (Patient.gender is null) then 'unknown' else Patient.gender\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BBMRI_STRAT_CUSTODIAN_STRATIFIER";
        let expected_result = "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BBMRI_STRAT_DIAGNOSIS_STRATIFIER";
        let expected_result = "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BBMRI_STRAT_AGE_STRATIFIER";
        let expected_result = "define AgeClass:\n     (AgeInYears() div 10) * 10\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BBMRI_STRAT_DEF_SPECIMEN";
        let expected_result = "define Specimen:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BBMRI_STRAT_DEF_IN_INITIAL_POPULATION";
        let expected_result = "define InInitialPopulation:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "EXLIQUID_CQL";
        let expected_result = "define ExliquidSpecimen:\nif InInitialPopulation then [Specimen] S\n where S.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen'\ndefine retrieveCondition:\n First(from [Condition] C\n  return C.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\ndefine Diagnosis:\n  if (retrieveCondition is null) then 'unknown' else retrieveCondition\ndefine function SampleType(specimen FHIR.Specimen):\n  specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "EXLIQUID_ALIQUOTS_CQL";
        let expected_result = "define Aliquot:\nif InInitialPopulation then [Specimen] S\n where exists S.collection.quantity.value and exists S.parent.reference and S.container.specimenQuantity.value > 0 define AliquotGroupReferences: flatten Aliquot S return S.parent.reference define AliquotGroupWithAliquot: [Specimen] S where not (S.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen') and not exists S.collection.quantity.value and not exists S.container.specimenQuantity.value and AliquotGroupReferences contains 'Specimen/' + S.id define PrimarySampleReferences: flatten AliquotGroupWithAliquot S return S.parent.reference define ExliquidSpecimenWithAliquot: from [Specimen] PrimarySample where PrimarySample.identifier.system contains 'http://dktk.dkfz.de/fhir/sid/exliquid-specimen'   and PrimarySampleReferences contains 'Specimen/' + PrimarySample.id    define retrieveCondition:   First(from [Condition] C   return C.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())    define Diagnosis:   if (retrieveCondition is null) then 'unknown' else retrieveCondition define function SampleType(specimen FHIR.Specimen): specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "EXLIQUID_STRAT_DEF_IN_INITIAL_POPULATION";
        let expected_result = "define InInitialPopulation:\n   exists ExliquidSpecimen and \n\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "EXLIQUID_STRAT_W_ALIQUOTS";
        let expected_result = "define InInitialPopulation: exists ExliquidSpecimenWithAliquot and \n\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "INVALID_KEY";
        let expected_result = "INVALID_KEY";
        assert_eq!(replace_cql(decoded_library), expected_result);
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
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();

        // Check that the obfuscated JSON can be parsed and has the same structure as the original JSON
        let _: MeasureReport = serde_json::from_str(&obfuscated_json).unwrap();

        // Check that the obfuscated JSON is different from the original JSON
        assert_ne!(obfuscated_json, EXAMPLE_MEASURE_REPORT_BBMRI);

        // Check that obfuscating the same JSON twice with the same obfuscation cache gives the same result
        let obfuscated_json_2 = obfuscate_counts_mr(EXAMPLE_MEASURE_REPORT_BBMRI, &mut obf_cache, false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            EPSILON,
            ROUNDING_STEP,).unwrap();
        assert_eq!(obfuscated_json, obfuscated_json_2);
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
            EPSILON,
            ROUNDING_STEP,
        )
        .unwrap();

        // Check that the obfuscated JSON can be parsed and has the same structure as the original JSON
        let _: MeasureReport = serde_json::from_str(&obfuscated_json).unwrap();

        dbg!(&obfuscated_json);

        // Check that the obfuscated JSON is different from the original JSON
        assert_ne!(obfuscated_json, EXAMPLE_MEASURE_REPORT_DKTK);

        // Check that obfuscating the same JSON twice with the same obfuscation cache gives the same result
        let obfuscated_json_2 = obfuscate_counts_mr(EXAMPLE_MEASURE_REPORT_DKTK, &mut obf_cache, false,
            1,
            DELTA_PATIENT,
            DELTA_SPECIMEN,
            DELTA_DIAGNOSIS,
            DELTA_PROCEDURES,
            DELTA_MEDICATION_STATEMENTS,
            EPSILON,
            ROUNDING_STEP,).unwrap();
        assert_eq!(obfuscated_json, obfuscated_json_2);
    }

}
