use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

pub(crate) static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("icd10", "http://hl7.org/fhir/sid/icd-10"),
        ("icd10gm", "http://fhir.de/CodeSystem/dimdi/icd-10-gm"),
        ("icd10gmnew", "http://fhir.de/CodeSystem/bfarm/icd-10-gm"),
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
    ])
});

pub(crate) static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("body_weight", "29463-7"),
        ("bmi", "39156-5"),
        ("smoking_status", "72166-2"),
    ])
});

pub(crate) static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from([
        ("diagnosis", vec!["icd10", "icd10gm", "icd10gmnew"]),
        ("body_weight", vec!["loinc"]),
        ("bmi", vec!["loinc"]),
        ("smoking_status", vec!["loinc"]),
        ("sample_kind", vec!["SampleMaterialType"]),
        ("storage_temperature", vec!["StorageTemperature"]),
        ("fasting_status", vec!["FastingStatus"]),
    ])
});

pub(crate) static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
        (
            ("diagnosis", CriterionRole::Query),
            "((exists[Condition: Code '{{C}}' from {{A1}}]) or (exists[Condition: Code '{{C}}' from {{A2}}]) or (exists[Condition: Code '{{C}}' from {{A3}}])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains '{{C}}'))",
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
    ])
});

pub(crate) static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> = LazyLock::new(|| {
    IndexSet::from(["icd10", "SampleMaterialType"])
});

pub(crate) static CQL_TEMPLATE: LazyLock<&'static str> = LazyLock::new(|| {
    include_str!("template.cql")
});

pub(crate) static BODY: LazyLock<&'static str> = LazyLock::new(|| {
    include_str!("body.json")
});

pub(crate) static SAMPLE_TYPE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    HashMap::from([
        (
            "blood-plasma",
            vec![
                "plasma-edta",
                "plasma-citrat",
                "plasma-heparin",
                "plasma-cell-free",
                "plasma-other",
                "plasma",
            ],
        ),
        ("blood-serum", vec!["serum"]),
        (
            "tissue-ffpe",
            vec![
                "tumor-tissue-ffpe",
                "normal-tissue-ffpe",
                "other-tissue-ffpe",
                "tissue-formalin",
            ],
        ),
        (
            "tissue-frozen",
            vec![
                "tumor-tissue-frozen",
                "normal-tissue-frozen",
                "other-tissue-frozen",
            ],
        ),
        ("dna", vec!["cf-dna", "g-dna"]),
        ("tissue-other", vec!["tissue-paxgene-or-else", "tissue"]),
        ("derivative-other", vec!["derivative"]),
        ("liquid-other", vec!["liquid"]),
    ])
});
