use std::collections::HashMap;

use indexmap::IndexSet;

use super::{CriterionRole, Project, ProjectName};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Dktk;

// TODO: Include entries from shared
impl Project for Dktk {
    fn append_code_lists(&self, _map: &mut HashMap<&'static str, &'static str>) { }

    fn append_observation_loinc_codes(&self, _map: &mut HashMap<&'static str, &'static str>) { }
    
    fn append_criterion_code_lists(&self, _map: &mut HashMap<(&str, &ProjectName), Vec<&str>>) { }
    
    fn append_cql_snippets(&self, _map: &mut HashMap<(&str, CriterionRole, &ProjectName), &str>) { }
    
    fn append_mandatory_code_lists(&self, map: &mut HashMap<&ProjectName, IndexSet<&str>>) {
        let mut set = map.remove(self.name()).unwrap_or(IndexSet::new());
        for value in ["icd10", "SampleMaterialType", "loinc"] {
            set.insert(value);
        }
        map.insert(self.name(), set);
    }
    
    fn append_cql_templates(&self, map: &mut HashMap<&ProjectName, &str>) {
        //map.insert(&Self, include_str!("template.cql"));
    }
    
    fn name(&self) -> &'static ProjectName {
        &ProjectName::Dktk
    }
}