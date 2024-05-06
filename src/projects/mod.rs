use std::{collections::HashMap, hash::Hash};

use indexmap::IndexSet;
use once_cell::sync::Lazy;

mod shared;

#[cfg(feature="bbmri")]
pub(crate) mod bbmri;

#[cfg(feature="dktk")]
pub(crate) mod dktk;

pub(crate) trait Project: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Hash {
    fn append_code_lists(&self, _map: &mut HashMap<&'static str, &'static str>);
    fn append_observation_loinc_codes(&self, _map: &mut HashMap<&'static str, &'static str>);
    fn append_criterion_code_lists(&self, _map: &mut HashMap<(&str, &ProjectName), Vec<&str>>);
    fn append_cql_snippets(&self, _map: &mut HashMap<(&str, CriterionRole, &ProjectName), &str>);
    fn append_mandatory_code_lists(&self, map: &mut HashMap<&ProjectName, IndexSet<&str>>);
    fn append_cql_templates(&self, map: &mut HashMap<&ProjectName, &str>);
    fn name(&self) -> &'static ProjectName;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum ProjectName {
    #[cfg(feature="bbmri")]
    Bbmri,
    #[cfg(feature="dktk")]
    Dktk,
    NotSpecified
}

// impl Display for ProjectName {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let name = match self {
//             ProjectName::Bbmri => "bbmri",
//             ProjectName::Dktk => "dktk"
//         };
//         write!(f, "{name}")
//     }
// }

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum CriterionRole {
    Query,
    Filter,
}

// code lists with their names
pub static CODE_LISTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();

    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_code_lists(&mut map);

    map
});

pub static OBSERVATION_LOINC_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    
    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_observation_loinc_codes(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_observation_loinc_codes(&mut map);

    map
});

//workarounds to search for subtypes and older codes of types
pub static SAMPLE_TYPE_WORKAROUNDS: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    
    #[cfg(feature="bbmri")]
    bbmri::append_sample_type_workarounds(&mut map);

    #[cfg(feature="dktk")]
    dktk::append_sample_type_workarounds(&mut map);

    map
});

// code lists needed depending on the criteria selected
pub static CRITERION_CODE_LISTS: Lazy<HashMap<(&str, &ProjectName), Vec<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_criterion_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_criterion_code_lists(&mut map);

    map
});

// CQL snippets depending on the criteria
pub static CQL_SNIPPETS: Lazy<HashMap<(&str, CriterionRole, &ProjectName), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_cql_snippets(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_cql_snippets(&mut map);

    map
});

pub static MANDATORY_CODE_SYSTEMS: Lazy<HashMap<&ProjectName, IndexSet<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_mandatory_code_lists(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_mandatory_code_lists(&mut map);

    map
});

pub static CQL_TEMPLATES: Lazy<HashMap<&ProjectName, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature="bbmri")]
    bbmri::Bbmri.append_cql_templates(&mut map);

    #[cfg(feature="dktk")]
    dktk::Dktk.append_cql_templates(&mut map);

    map
});
