use std::collections::HashMap;

use indexmap::IndexSet;

use super::{Project, CriterionRole};

const PROJECT: Project = Project::Bbmri;

pub fn append_code_lists(_map: &mut HashMap<&'static str, &'static str>) { }

pub fn append_observation_loinc_codes(_map: &mut HashMap<&'static str, &'static str>) { }

pub fn append_criterion_code_lists(map: &mut HashMap<(&str, Project), Vec<&str>>) {
    for (key, value) in 
    [
        ("diagnosis", vec!["icd10", "icd10gm"]),
        ("body_weight", vec!["loinc"]),
        ("bmi", vec!["loinc"]),
        ("smoking_status", vec!["loinc"]),
        ("sample_kind", vec!["SampleMaterialType"]),
        ("storage_temperature", vec!["StorageTemperature"]),
        ("fasting_status", vec!["FastingStatus"]),
    ] {
        map.insert(
            (key, PROJECT),
            value
        );
    }
}

pub fn append_cql_snippets(map: &mut HashMap<(&str, CriterionRole, Project), &str>) {
    for (key, value) in
    [
        (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
        (
            ("diagnosis", CriterionRole::Query),
            "((exists[Condition: Code '{{C}}' from {{A1}}]) or (exists[Condition: Code '{{C}}' from {{A2}}])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains '{{C}}'))",
        ),
        (("diagnosis_old", CriterionRole::Query), " exists [Condition: Code '{{C}}' from {{A1}}]"),
        (
            ("date_of_diagnosis", CriterionRole::Query),
            "exists from [Condition] C\nwhere FHIRHelpers.ToDateTime(C.onset) between {{D1}} and {{D2}}",
        ),
        (
            ("diagnosis_age_donor", CriterionRole::Query),
            "exists from [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling({{D1}}) and Ceiling({{D2}})",
        ),
        (("donor_age", CriterionRole::Query), " AgeInYears() between Ceiling({{D1}}) and Ceiling({{D2}})"),
        (
            ("observationRange", CriterionRole::Query),
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value between {{D1}} and {{D2}}",
        ),
        (
            ("body_weight", CriterionRole::Query),
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg' and (O.value as Quantity) > {{D2}} 'kg')",
        ),
        (
            ("bmi", CriterionRole::Query),
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg/m2' and (O.value as Quantity) > {{D2}} 'kg/m2')",
        ),
        (("sample_kind", CriterionRole::Query), " exists [Specimen: Code '{{C}}' from {{A1}}]"),
        (("sample_kind", CriterionRole::Filter), " (S.type.coding.code contains '{{C}}')"),

        (
            ("storage_temperature", CriterionRole::Filter),
            "(S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains '{{C}}')",
        ),
        (
            ("sampling_date", CriterionRole::Filter),
            "(FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}}) ",
        ),
        (
            ("fasting_status", CriterionRole::Filter),
            "(S.collection.fastingStatus.coding.code contains '{{C}}') ",
        ),
        (
            ("sampling_date", CriterionRole::Query),
            "exists from [Specimen] S\nwhere FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}} ",
        ),
        (
            ("fasting_status", CriterionRole::Query),   
            "exists from [Specimen] S\nwhere S.collection.fastingStatus.coding.code contains '{{C}}' ",
        ),
        (
            ("storage_temperature", CriterionRole::Query), 
            "exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code '{{C}}' from {{A1}}) ",
        ),
        (
            ("smoking_status", CriterionRole::Query), 
            "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}' ",
        ),
    ] {
        map.insert(
            (key.0, key.1, PROJECT),
            value
        );
    }
}

pub fn append_mandatory_code_lists(map: &mut HashMap<Project, IndexSet<&str>>) {
    let mut set = map.remove(&PROJECT).unwrap_or(IndexSet::new());
    for value in ["icd10", "SampleMaterialType"] {
        set.insert(value);
    }
    map.insert(PROJECT, set);
}

pub(crate) fn append_cql_templates(map: &mut HashMap<Project, &str>) {
    map.insert(PROJECT, include_str!("../../resources/template_bbmri.cql"));
}