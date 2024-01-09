use std::collections::HashMap;

use indexmap::IndexSet;

use super::{Project, CriterionRole};

pub fn append_code_lists(map: &mut HashMap<&'static str, &'static str>) {
    map.extend(
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
    ]);
}

pub fn append_observation_loinc_codes(map: &mut HashMap<&'static str, &'static str>) {
    map.extend(
    [
        ("body_weight", "29463-7"),
        ("bmi", "39156-5"),
        ("smoking_status", "72166-2"),
    ]);
}

pub fn append_criterion_code_lists(map: &mut HashMap<(&str, Project), Vec<&str>>) { }

pub fn append_cql_snippets(map: &mut HashMap<(&str, CriterionRole, Project), &str>) { }

pub fn append_mandatory_code_lists(map: &mut HashMap<Project, IndexSet<&str>>) { }

pub(crate) fn append_cql_templates(map: &mut HashMap<Project, &str>) { }