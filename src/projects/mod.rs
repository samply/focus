use std::{collections::HashMap, hash::Hash, str::FromStr};

use indexmap::IndexSet;

use crate::errors::FocusError;

mod bbmri;
mod dktk;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum CriterionRole {
    Query,
    Filter,
}

pub enum Project {
    Bbmri,
    Dktk,
}

impl FromStr for Project {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bbmri" => Ok(Project::Bbmri),
            "dktk" => Ok(Project::Dktk),
            _ => Err(FocusError::UnknownProject(s.to_string())),
        }
    }
}

pub fn get_code_lists(project: &Project) -> &'static HashMap<&'static str, &'static str> {
    match project {
        Project::Bbmri => &bbmri::CODE_LISTS,
        Project::Dktk => &dktk::CODE_LISTS,
    }
}

pub fn get_observation_loinc_codes(
    project: &Project,
) -> &'static HashMap<&'static str, &'static str> {
    match project {
        Project::Bbmri => &bbmri::OBSERVATION_LOINC_CODES,
        Project::Dktk => &dktk::OBSERVATION_LOINC_CODES,
    }
}

pub fn get_sample_type_workarounds(
    project: &Project,
) -> &'static HashMap<&'static str, Vec<&'static str>> {
    match project {
        Project::Bbmri => &bbmri::SAMPLE_TYPE_WORKAROUNDS,
        Project::Dktk => &dktk::SAMPLE_TYPE_WORKAROUNDS,
    }
}

pub fn get_criterion_code_lists(
    project: &Project,
) -> &'static HashMap<&'static str, Vec<&'static str>> {
    match project {
        Project::Bbmri => &bbmri::CRITERION_CODE_LISTS,
        Project::Dktk => &dktk::CRITERION_CODE_LISTS,
    }
}

pub fn get_cql_snippets(
    project: &Project,
) -> &'static HashMap<(&'static str, CriterionRole), &'static str> {
    match project {
        Project::Bbmri => &bbmri::CQL_SNIPPETS,
        Project::Dktk => &dktk::CQL_SNIPPETS,
    }
}

pub fn get_mandatory_code_lists(project: &Project) -> &'static IndexSet<&'static str> {
    match project {
        Project::Bbmri => &bbmri::MANDATORY_CODE_LISTS,
        Project::Dktk => &dktk::MANDATORY_CODE_LISTS,
    }
}

pub fn get_cql_template(project: &Project) -> &'static str {
    match project {
        Project::Bbmri => include_str!("bbmri/template.cql"),
        Project::Dktk => include_str!("dktk/template.cql"),
    }
}

pub fn get_body(project: &Project) -> &'static str {
    match project {
        Project::Bbmri => include_str!("bbmri/body.json"),
        Project::Dktk => include_str!("dktk/body.json"),
    }
}
