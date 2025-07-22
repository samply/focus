use crate::projects::shared::Shared;

use std::collections::HashMap;

use indexmap::IndexSet;

use super::{CriterionRole, Project, ProjectName};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Dktk;

// TODO: Include entries from shared
impl Project for Dktk {
    fn append_code_lists(&self, map: &mut HashMap<&'static str, &'static str>) {
        Shared::append_code_lists(&Shared, map)
    }

    fn append_observation_loinc_codes(&self, map: &mut HashMap<&'static str, &'static str>) {
        Shared::append_observation_loinc_codes(&Shared, map)
    }

    fn append_criterion_code_lists(&self, _map: &mut HashMap<&str, Vec<&str>>) {}

    fn append_cql_snippets(&self, _map: &mut HashMap<(&str, CriterionRole), &str>) {}

    fn append_mandatory_code_lists(&self, map: &mut IndexSet<&str>) {
        //let mut set = map.remove(self.name()).unwrap_or(IndexSet::new());
        for value in ["icd10", "SampleMaterialType", "loinc"] {
            map.insert(value);
        }
        //map.insert(self.name(), set);
    }

    fn append_cql_template(&self, _template: &mut String) {
        //include_str!("template.cql")
    }

    fn name(&self) -> &'static ProjectName {
        &ProjectName::Dktk
    }

    fn append_body(&self, str: &mut String) {
        str.push_str(include_str!("body.json"));
    }

    fn append_sample_type_workarounds(&self, _map: &mut HashMap<&str, Vec<&str>>) {
        //none
    }
}
