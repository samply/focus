use std::collections::HashMap;

use indexmap::IndexSet;

use super::{Project, CriterionRole};

const PROJECT: Project = Project::Dktk;

pub fn append_code_lists(_map: &mut HashMap<&'static str, &'static str>) { }

pub fn append_observation_loinc_codes(_map: &mut HashMap<&'static str, &'static str>) { }

pub fn append_criterion_code_lists(_map: &mut HashMap<(&str, Project), Vec<&str>>) { }

pub fn append_cql_snippets(_map: &mut HashMap<(&str, CriterionRole, Project), &str>) { }

pub fn append_mandatory_code_lists(map: &mut HashMap<Project, IndexSet<&str>>) {
    let mut set = map.remove(&PROJECT).unwrap_or(IndexSet::new());
    for value in ["icd10", "SampleMaterialType", "loinc"] {
        set.insert(value);
    }
    map.insert(PROJECT, set);
}

pub(crate) fn append_cql_templates(map: &mut HashMap<Project, &str>) {
    //map.insert(PROJECT, include_str!("../../resources/template_dktk.cql"));
}