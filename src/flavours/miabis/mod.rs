use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

pub static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("icd10", "http://hl7.org/fhir/sid/icd-10"),
        (
            "MiabisDetailedSampleType",
            "https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs",
        ),
    ])
});

pub static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(HashMap::new);

pub static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("diagnosis", vec!["icd10"]),
            ("sample_kind", vec!["MiabisDetailedSampleType"]),
        ])
    });

pub static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
            (
                ("diagnosis", CriterionRole::Query),
                "exists[Condition: Code '{{C}}' from {{A1}}]",
            ),
            (
                ("date_of_diagnosis", CriterionRole::Query),
                "exists from [Condition] C\nwhere FHIRHelpers.ToDateTime(C.onset) between {{D1}} and {{D2}}",
            ),
            (
                ("diagnosis_age_donor", CriterionRole::Query),
                "exists from [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling({{D1}}) and Ceiling({{D2}})",
            ),
            (
                ("donor_age", CriterionRole::Query),
                "AgeInYears() between Ceiling({{D1}}) and Ceiling({{D2}})",
            ),
            (
                ("sample_kind", CriterionRole::Query),
                "exists [Specimen: Code '{{C}}' from {{A1}}]",
            ),
            (
                ("sample_kind", CriterionRole::Filter),
                "(S.type.coding.where(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains '{{C}}')",
            ),
            (
                ("storage_temperature", CriterionRole::Query),
                "exists from [Specimen] S where (\
                    exists from S.processing P where (\
                        exists from P.extension E where \
                            E.url = 'https://fhir.bbmri-eric.eu/StructureDefinition/miabis-sample-storage-temperature-extension' \
                            and (E.value as CodeableConcept).coding.code contains '{{C}}'\
                    )\
                )",
            ),
            (
                ("storage_temperature", CriterionRole::Filter),
                "(exists from S.processing P where (\
                    exists from P.extension E where \
                        E.url = 'https://fhir.bbmri-eric.eu/StructureDefinition/miabis-sample-storage-temperature-extension' \
                        and (E.value as CodeableConcept).coding.code contains '{{C}}'\
                ))",
            ),
            (
                ("sampling_date", CriterionRole::Query),
                "exists from [Specimen] S\nwhere FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}}",
            ),
            (
                ("sampling_date", CriterionRole::Filter),
                "(FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}})",
            ),
        ])
    });

pub static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> =
    LazyLock::new(|| IndexSet::from(["icd10", "MiabisDetailedSampleType"]));

pub static CODE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("whole-blood", vec!["WholeBlood"]),
            ("buffy-coat", vec!["BuffyCoat"]),
            ("blood-plasma", vec!["Plasma"]),
            ("blood-serum", vec!["Serum"]),
            ("dna", vec!["DNA"]),
            ("rna", vec!["RNA"]),
            ("peripheral-blood-cells-vital", vec!["PBMC"]),
            ("tissue-frozen", vec!["TissueFreshFrozen"]),
            ("tissue-ffpe", vec!["TissueFixed"]),
            ("bone-marrow", vec!["BoneMarrowAspirate"]),
            ("csf-liquor", vec!["CerebrospinalFluid"]),
            ("urine", vec!["Urine"]),
            ("saliva", vec!["Saliva"]),
            ("stool-faeces", vec!["Faeces"]),
            ("liquid-other", vec!["LiquidBiopsy", "Sputum"]),
            ("temperatureRoom", vec!["RT"]),
            ("temperature2to10", vec!["2to10"]),
            ("four_degrees", vec!["2to10"]),
            ("temperature-18to-35", vec!["-18to-35"]),
            ("temperature-60to-85", vec!["-60to-85"]),
            ("temperatureGN", vec!["LN"]),
            ("temperatureLN", vec!["LN"]),
            ("temperatureOther", vec!["Other"]),
        ])
    });
