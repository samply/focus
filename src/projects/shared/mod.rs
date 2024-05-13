use std::collections::HashMap;

use indexmap::IndexSet;

use super::{CriterionRole, Project, ProjectName};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Shared;

impl Project for Shared {
    fn append_code_lists(&self, map: &mut HashMap<&'static str, &'static str>) {
        map.extend(
        [
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
        map.extend(
        [
            ("body_weight", "29463-7"),
            ("bmi", "39156-5"),
            ("smoking_status", "72166-2"),
        ]);
    }
    
    fn append_criterion_code_lists(&self, _map: &mut HashMap<&str, Vec<&str>>) {
        // none
    }
    
    fn append_cql_snippets(&self, _map: &mut HashMap<(&str, CriterionRole), &str>) {
        // none
    }
    
    fn append_mandatory_code_lists(&self, _set: &mut IndexSet<&str>) {
        // none
    }
    
    fn append_cql_template(&self, _template:  &mut String) {
        // none
    }

    fn append_body(&self, _body:  &mut String) {
        // none
    }

    fn name(&self) -> &'static ProjectName {
        &ProjectName::NotSpecified
    }
    
    fn append_sample_type_workarounds(&self, _map: &mut HashMap<&str, Vec<&str>>) {
        //none
    }
    

}