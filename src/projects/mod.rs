use std::{collections::HashMap, fmt::Display};

use indexmap::IndexSet;
use once_cell::sync::Lazy;

mod common;
#[cfg(feature="bbmri")]
mod bbmri;
#[cfg(feature="dktk")]
mod dktk;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum Project {
    #[cfg(feature="bbmri")]
    Bbmri,
    #[cfg(feature="dktk")]
    Dktk,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Project::Bbmri => "bbmri",
            Project::Dktk => "dktk",
        };
        write!(f, "{name}")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum CriterionRole {
    Query,
    Filter,
}

// code lists with their names
pub static CODE_LISTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    common::append_code_lists(&mut map);

    #[cfg(feature="bbmri")]
    bbmri::append_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_code_lists(&mut map);

    map
});

pub static OBSERVATION_LOINC_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    common::append_observation_loinc_codes(&mut map);
    
    #[cfg(feature="bbmri")]
    bbmri::append_observation_loinc_codes(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_observation_loinc_codes(&mut map);

    map
});

// code lists needed depending on the criteria selected
pub static CRITERION_CODE_LISTS: Lazy<HashMap<(&str, Project), Vec<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    common::append_criterion_code_lists(&mut map);

    #[cfg(feature="bbmri")]
    bbmri::append_criterion_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_criterion_code_lists(&mut map);

    map
});

// CQL snippets depending on the criteria
pub static CQL_SNIPPETS: Lazy<HashMap<(&str, CriterionRole, Project), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    common::append_cql_snippets(&mut map);

    #[cfg(feature="bbmri")]
    bbmri::append_cql_snippets(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_cql_snippets(&mut map);

    map
});

pub static MANDATORY_CODE_SYSTEMS: Lazy<HashMap<Project, IndexSet<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    common::append_mandatory_code_lists(&mut map);

    #[cfg(feature="bbmri")]
    bbmri::append_mandatory_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_mandatory_code_lists(&mut map);

    map
});

pub static CQL_TEMPLATES: Lazy<HashMap<Project, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    common::append_cql_templates(&mut map);

    #[cfg(feature="bbmri")]
    bbmri::append_cql_templates(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_cql_templates(&mut map);

    map
});
