use std::{collections::HashMap, hash::Hash, str::FromStr};

use indexmap::IndexSet;

use crate::errors::FocusError;

mod bbmri;
mod cce;
mod dhki;
mod dktk;
mod nngm;
mod pscc;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum CriterionRole {
    Query,
    Filter,
}

pub enum Project {
    Bbmri,
    Dktk,
    Cce,
    Dhki,
    Nngm,
	Pscc,
}

impl FromStr for Project {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bbmri" => Ok(Project::Bbmri),
            "dktk" => Ok(Project::Dktk),
            "cce" => Ok(Project::Cce),
            "nngm" => Ok(Project::Nngm),
            "dhki" => Ok(Project::Dhki),
            "pscc" => Ok(Project::Pscc),
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
            Project::Dhki => &dhki::CODE_LISTS,
            Project::Nngm => &nngm::CODE_LISTS,
            Project::Pscc => &pscc::CODE_LISTS,
        }
    }

    pub fn get_observation_loinc_codes(&self) -> &'static HashMap<&'static str, &'static str> {
        match self {
            Project::Bbmri => &bbmri::OBSERVATION_LOINC_CODES,
            Project::Dktk => &dktk::OBSERVATION_LOINC_CODES,
            Project::Cce => &cce::OBSERVATION_LOINC_CODES,
            Project::Dhki => &dhki::OBSERVATION_LOINC_CODES,
            Project::Nngm => &nngm::OBSERVATION_LOINC_CODES,
            Project::Pscc => &pscc::OBSERVATION_LOINC_CODES,
        }
    }

    pub fn get_sample_type_workarounds(&self) -> &'static HashMap<&'static str, Vec<&'static str>> {
        match self {
            Project::Bbmri => &bbmri::SAMPLE_TYPE_WORKAROUNDS,
            Project::Dktk => &dktk::SAMPLE_TYPE_WORKAROUNDS,
            Project::Cce => &cce::SAMPLE_TYPE_WORKAROUNDS,
            Project::Dhki => &dhki::SAMPLE_TYPE_WORKAROUNDS,
            Project::Nngm => &nngm::SAMPLE_TYPE_WORKAROUNDS,
            Project::Pscc => &pscc::SAMPLE_TYPE_WORKAROUNDS,
        }
    }

    pub fn get_criterion_code_lists(&self) -> &'static HashMap<&'static str, Vec<&'static str>> {
        match self {
            Project::Bbmri => &bbmri::CRITERION_CODE_LISTS,
            Project::Dktk => &dktk::CRITERION_CODE_LISTS,
            Project::Cce => &cce::CRITERION_CODE_LISTS,
            Project::Dhki => &dhki::CRITERION_CODE_LISTS,
            Project::Nngm => &nngm::CRITERION_CODE_LISTS,
            Project::Pscc => &pscc::CRITERION_CODE_LISTS,
        }
    }

    pub fn get_cql_snippets(
        &self,
    ) -> &'static HashMap<(&'static str, CriterionRole), &'static str> {
        match self {
            Project::Bbmri => &bbmri::CQL_SNIPPETS,
            Project::Dktk => &dktk::CQL_SNIPPETS,
            Project::Cce => &cce::CQL_SNIPPETS,
            Project::Dhki => &dhki::CQL_SNIPPETS,
            Project::Nngm => &nngm::CQL_SNIPPETS,
            Project::Pscc => &pscc::CQL_SNIPPETS,
        }
    }

    pub fn get_mandatory_code_lists(&self) -> &'static IndexSet<&'static str> {
        match self {
            Project::Bbmri => &bbmri::MANDATORY_CODE_LISTS,
            Project::Dktk => &dktk::MANDATORY_CODE_LISTS,
            Project::Cce => &cce::MANDATORY_CODE_LISTS,
            Project::Dhki => &dhki::MANDATORY_CODE_LISTS,
            Project::Nngm => &nngm::MANDATORY_CODE_LISTS,
            Project::Pscc => &pscc::MANDATORY_CODE_LISTS,
        }
    }

    pub fn get_cql_template(&self) -> &'static str {
        match self {
            Project::Bbmri => include_str!("bbmri/template.cql"),
            Project::Dktk => include_str!("dktk/template.cql"),
            Project::Cce => include_str!("cce/template.cql"),
            Project::Dhki => include_str!("dhki/template.cql"),
            Project::Nngm => include_str!("nngm/template.cql"),
            Project::Pscc => include_str!("pscc/template.cql"),
        }
    }

    pub fn get_body(&self) -> &'static str {
        match self {
            Project::Bbmri => include_str!("bbmri/body.json"),
            Project::Dktk => include_str!("dktk/body.json"),
            Project::Cce => include_str!("cce/body.json"),
            Project::Dhki => include_str!("dhki/body.json"),
            Project::Nngm => include_str!("nngm/body.json"),
            Project::Pscc => include_str!("pscc/body.json"),
        }
    }
}
