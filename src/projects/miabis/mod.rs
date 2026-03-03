use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

// CodeSystem URIs for MIABIS-on-FHIR 1.0.0
// Hosted at https://fhir.bbmri-eric.eu (Czech BBMRI-cz IG)

pub static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("icd10", "http://hl7.org/fhir/sid/icd-10"),
        (
            "MiabisDetailedSampleType",
            "https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs",
        ),
    ])
});

// MIABIS-on-FHIR 1.0.0 does not define observation-based criteria.
pub static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(HashMap::new);

pub static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("diagnosis", vec!["icd10"]),
            ("sample_kind", vec!["MiabisDetailedSampleType"]),
            // storage_temperature uses extension URL matching, no declared codesystem needed
        ])
    });

pub static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            // ── Patient-level criteria ────────────────────────────────────────────────
            (
                ("gender", CriterionRole::Query),
                "Patient.gender = '{{C}}'",
            ),
            // MIABIS uses ICD-10 only — no German ICD-10-GM variants,
            // no SampleDiagnosis specimen extension.
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

            // ── Specimen-level criteria ───────────────────────────────────────────────
            //
            // sample_kind: uses the MIABIS detailed sample type CodeSystem.
            // SAMPLE_TYPE_WORKAROUNDS maps canonical Sample Locator codes
            // (e.g. "blood-plasma") to MIABIS codes (e.g. "Plasma").
            (
                ("sample_kind", CriterionRole::Query),
                "exists [Specimen: Code '{{C}}' from {{A1}}]",
            ),
            (
                ("sample_kind", CriterionRole::Filter),
                "(S.type.coding.where(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').code contains '{{C}}')",
            ),

            // storage_temperature: MIABIS places the extension in
            // Specimen.processing[].extension, not Specimen.extension directly.
            // Nested 'from P.extension' avoids the Blaze "name cannot be null" error.
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

            // sampling_date: same path as de.bbmri.fhir
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

// icd10 and MiabisDetailedSampleType are always declared in the generated CQL
// so that the template's SampleType function and DiagnosisCode function can
// reference them even when no search criteria are specified.
pub static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> =
    LazyLock::new(|| IndexSet::from(["icd10", "MiabisDetailedSampleType"]));

// Maps canonical Sample Locator codes to the MIABIS-on-FHIR 1.0.0 codes
// stored in the data (miabis-detailed-samply-type-cs).
// When a user searches for e.g. "blood-plasma", focus also filters for "Plasma".
pub static SAMPLE_TYPE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("whole-blood",                 vec!["WholeBlood"]),
            ("buffy-coat",                  vec!["BuffyCoat"]),
            ("blood-plasma",                vec!["Plasma"]),
            ("blood-serum",                 vec!["Serum"]),
            ("dna",                         vec!["DNA"]),
            ("rna",                         vec!["RNA"]),
            ("peripheral-blood-cells-vital",vec!["PBMC"]),
            ("tissue-frozen",               vec!["TissueFreshFrozen"]),
            ("tissue-ffpe",                 vec!["TissueFixed"]),
            ("bone-marrow",                 vec!["BoneMarrowAspirate"]),
            ("csf-liquor",                  vec!["CerebrospinalFluid"]),
            ("urine",                       vec!["Urine"]),
            ("saliva",                      vec!["Saliva"]),
            ("stool-faeces",                vec!["Faeces"]),
            ("liquid-other",                vec!["LiquidBiopsy", "Sputum"]),
        ])
    });

// Maps canonical Sample Locator storage temperature codes (sent by Lens) to the
// MIABIS-on-FHIR 1.0.0 codes stored in the data
// (miabis-storage-temperature-cs, hosted at https://fhir.bbmri-eric.eu).
//
// Lens code             MIABIS code   Notes
// ─────────────────────────────────────────────────────────────────────────────
// temperatureRoom       RT            Room temperature
// temperature2to10      2to10         2–10 °C
// four_degrees          2to10         4 °C is within the MIABIS 2–10 °C band;
//                                     MIABIS has no dedicated 4 °C code
// temperature-18to-35   -18to-35      −18 to −35 °C
// temperature-60to-85   -60to-85      −60 to −85 °C
// temperatureGN         LN            Gaseous/vapour-phase nitrogen (−150 to
//                                     −196 °C); mapped to MIABIS LN (liquid
//                                     nitrogen, same range) — closest available
// temperatureLN         LN            Liquid nitrogen
// temperatureOther      Other         Any other temperature
//
// Note: "storage_temperature_uncharted" has no MIABIS equivalent and is left
// unmapped; queries for it will return zero results on MIABIS nodes, which is
// the correct behaviour (the information is genuinely absent in the data model).
pub static STORAGE_TEMPERATURE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("temperatureRoom",    vec!["RT"]),
            ("temperature2to10",   vec!["2to10"]),
            ("four_degrees",       vec!["2to10"]),
            ("temperature-18to-35",vec!["-18to-35"]),
            ("temperature-60to-85",vec!["-60to-85"]),
            ("temperatureGN",      vec!["LN"]),
            ("temperatureLN",      vec!["LN"]),
            ("temperatureOther",   vec!["Other"]),
        ])
    });
