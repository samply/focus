use std::{collections::HashMap, hash::Hash};

use indexmap::IndexSet;
use once_cell::sync::Lazy;

mod shared;

#[cfg(feature = "bbmri")]
pub(crate) mod bbmri;

#[cfg(feature = "dktk")]
pub(crate) mod dktk;

#[cfg(feature = "cce")]
pub(crate) mod cce;

pub(crate) trait Project: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + Hash {
    fn append_code_lists(&self, _map: &mut HashMap<&'static str, &'static str>);
    fn append_observation_loinc_codes(&self, _map: &mut HashMap<&'static str, &'static str>);
    fn append_criterion_code_lists(&self, _map: &mut HashMap<&str, Vec<&str>>);
    fn append_cql_snippets(&self, _map: &mut HashMap<(&str, CriterionRole), &str>);
    fn append_mandatory_code_lists(&self, set: &mut IndexSet<&str>);
    fn append_cql_template(&self, _template: &mut String);
    fn name(&self) -> &'static ProjectName;
    fn append_body(&self, _body: &mut String);
    fn append_sample_type_workarounds(&self, _map: &mut HashMap<&str, Vec<&str>>);
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum ProjectName {
    #[cfg(feature = "bbmri")]
    Bbmri,
    #[cfg(feature = "dktk")]
    Dktk,
    #[cfg(feature = "cce")]
    Cce,
    NotSpecified,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum CriterionRole {
    Query,
    Filter,
}

// code lists with their names
pub static CODE_LISTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_code_lists(&mut map);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_code_lists(&mut map);

    #[cfg(feature = "cce")]
    cce::Cce.append_code_lists(&mut map);

    map
});

pub static OBSERVATION_LOINC_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_observation_loinc_codes(&mut map);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_observation_loinc_codes(&mut map);

    #[cfg(feature = "cce")]
    cce::Cce.append_observation_loinc_codes(&mut map);

    map
});

//workarounds to search for subtypes and older codes of types
pub static SAMPLE_TYPE_WORKAROUNDS: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_sample_type_workarounds(&mut map);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_sample_type_workarounds(&mut map);

    #[cfg(feature = "cce")]
    cce::Cce.append_sample_type_workarounds(&mut map);

    map
});

// code lists needed depending on the criteria selected
pub static CRITERION_CODE_LISTS: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_criterion_code_lists(&mut map);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_criterion_code_lists(&mut map);

    #[cfg(feature = "cce")]
    cce::Cce.append_criterion_code_lists(&mut map);

    map
});

// CQL snippets depending on the criteria
pub static CQL_SNIPPETS: Lazy<HashMap<(&str, CriterionRole), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_cql_snippets(&mut map);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_cql_snippets(&mut map);

    #[cfg(feature = "cce")]
    cce::Cce.append_cql_snippets(&mut map);

    map
});

pub static MANDATORY_CODE_SYSTEMS: Lazy<IndexSet<&str>> = Lazy::new(|| {
    let mut set = IndexSet::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_mandatory_code_lists(&mut set);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_mandatory_code_lists(&mut set);

    #[cfg(feature = "cce")]
    cce::Cce.append_mandatory_code_lists(&mut set);

    set
});

pub static CQL_TEMPLATE: Lazy<&'static str> = Lazy::new(|| {
    let mut template = String::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_cql_template(&mut template);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_cql_template(&mut template);

    #[cfg(feature = "cce")]
    cce::Cce.append_cql_template(&mut template);

    template.leak()
});

pub static BODY: Lazy<&'static str> = Lazy::new(|| {
    let mut body = String::new();

    #[cfg(feature = "bbmri")]
    bbmri::Bbmri.append_body(&mut body);

    #[cfg(feature = "dktk")]
    dktk::Dktk.append_body(&mut body);

    #[cfg(feature = "cce")]
    cce::Cce.append_body(&mut body);

    body.leak()
});
