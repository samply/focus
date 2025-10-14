use std::{collections::HashMap, hash::Hash, str::FromStr};

use indexmap::IndexSet;

use crate::errors::FocusError;

mod bbmri;
mod cce;
mod dktk;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum CriterionRole {
    Query,
    Filter,
}

pub enum Project {
    Bbmri,
    Dktk,
    Cce,
}

impl FromStr for Project {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bbmri" => Ok(Project::Bbmri),
            "dktk" => Ok(Project::Dktk),
            "cce" => Ok(Project::Cce),
            _ => Err(FocusError::UnknownProject(s.to_string())),
        }
    }
}

impl Project {
    pub fn get_code_lists(&self) -> &'static HashMap<&'static str, &'static str> {
        match self {
            Project::Bbmri => &bbmri::CODE_LISTS,
            Project::Dktk => &dktk::CODE_LISTS,
            Project::Cce => &cce::CODE_LISTS,
        }
    }

    pub fn get_observation_loinc_codes(&self) -> &'static HashMap<&'static str, &'static str> {
        match self {
            Project::Bbmri => &bbmri::OBSERVATION_LOINC_CODES,
            Project::Dktk => &dktk::OBSERVATION_LOINC_CODES,
            Project::Cce => &cce::OBSERVATION_LOINC_CODES,
        }
    }

    pub fn get_criterion_code_lists(&self) -> &'static HashMap<&'static str, Vec<&'static str>> {
        match self {
            Project::Bbmri => &bbmri::CRITERION_CODE_LISTS,
            Project::Dktk => &dktk::CRITERION_CODE_LISTS,
            Project::Cce => &cce::CRITERION_CODE_LISTS,
        }
    }

    pub fn get_cql_snippets(
        &self,
    ) -> &'static HashMap<(&'static str, CriterionRole), &'static str> {
        match self {
            Project::Bbmri => &bbmri::CQL_SNIPPETS,
            Project::Dktk => &dktk::CQL_SNIPPETS,
            Project::Cce => &cce::CQL_SNIPPETS,
        }
    }

    pub fn get_mandatory_code_lists(&self) -> &'static IndexSet<&'static str> {
        match self {
            Project::Bbmri => &bbmri::MANDATORY_CODE_LISTS,
            Project::Dktk => &dktk::MANDATORY_CODE_LISTS,
            Project::Cce => &cce::MANDATORY_CODE_LISTS,
        }
    }

    pub fn get_cql_template(&self) -> &'static str {
        match self {
            Project::Bbmri => include_str!("bbmri/template.cql"),
            Project::Dktk => include_str!("dktk/template.cql"),
            Project::Cce => include_str!("cce/template.cql"),
        }
    }

    pub fn get_body(&self) -> &'static str {
        match self {
            Project::Bbmri => include_str!("bbmri/body.json"),
            Project::Dktk => include_str!("dktk/body.json"),
            Project::Cce => include_str!("cce/body.json"),
        }
    }
}
