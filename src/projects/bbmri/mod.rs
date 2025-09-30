use std::collections::HashMap;

use indexmap::IndexSet;

use super::{CriterionRole, Project, ProjectName};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Bbmri;

// TODO: Include entries from shared
impl Project for Bbmri {
    fn append_code_lists(&self, map: &mut HashMap<&'static str, &'static str>) {
        map.extend([
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
        ]);
    }

    fn append_observation_loinc_codes(&self, map: &mut HashMap<&'static str, &'static str>) {
        map.extend([
            ("body_weight", "29463-7"),
            ("bmi", "39156-5"),
            ("smoking_status", "72166-2"),
        ]);
    }

    fn append_criterion_code_lists(&self, map: &mut HashMap<&str, Vec<&str>>) {
        for (key, value) in [
            ("diagnosis", vec!["icd10", "icd10gm", "icd10gmnew"]),
            ("body_weight", vec!["loinc"]),
            ("bmi", vec!["loinc"]),
            ("smoking_status", vec!["loinc"]),
            ("sample_kind", vec!["SampleMaterialType"]),
            ("storage_temperature", vec!["StorageTemperature"]),
            ("fasting_status", vec!["FastingStatus"]),
        ] {
            map.insert(key, value);
        }
    }

    fn append_cql_snippets(&self, map: &mut HashMap<(&str, CriterionRole), &str>) {
        for (key, value) in
        [
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
        ] {
            map.insert(
                (key.0, key.1),
                value
            );
        }
    }

    fn append_mandatory_code_lists(&self, set: &mut IndexSet<&str>) {
        for value in ["icd10", "SampleMaterialType"] {
            set.insert(value);
        }
    }

    fn append_cql_template(&self, template: &mut String) {
        template.push_str(include_str!("template.cql"));
    }

    fn name(&self) -> &'static ProjectName {
        &ProjectName::Bbmri
    }

    fn append_body(&self, body: &mut String) {
        body.push_str(include_str!("body.json"));
    }

    fn append_sample_type_workarounds(&self, map: &mut HashMap<&str, Vec<&str>>) {
        for (key, value) in [
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
        ] {
            map.insert(key, value);
        }
    }
}
