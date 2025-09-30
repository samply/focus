use std::{collections::HashMap, hash::Hash};

use indexmap::IndexSet;
use once_cell::sync::Lazy;

#[cfg(feature = "bbmri")]
pub(crate) mod bbmri;

#[cfg(feature = "dktk")]
pub(crate) mod dktk;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum CriterionRole {
    Query,
    Filter,
}

// code lists with their names
pub static CODE_LISTS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();

    #[cfg(feature = "bbmri")]
    map.extend(bbmri::CODE_LISTS.iter().map(|(k, v)| (*k, *v)));

    #[cfg(feature = "dktk")]
    map.extend(dktk::CODE_LISTS.iter().map(|(k, v)| (*k, *v)));

    map
});

pub static OBSERVATION_LOINC_CODE: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();

    #[cfg(feature = "bbmri")]
    map.extend(bbmri::OBSERVATION_LOINC_CODES.iter().map(|(k, v)| (*k, *v)));

    #[cfg(feature = "dktk")]
    map.extend(dktk::OBSERVATION_LOINC_CODES.iter().map(|(k, v)| (*k, *v)));

    map
});

//workarounds to search for subtypes and older codes of types
pub static SAMPLE_TYPE_WORKAROUNDS: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, Vec<&'static str>> = HashMap::new();

    #[cfg(feature = "bbmri")]
    map.extend(bbmri::SAMPLE_TYPE_WORKAROUNDS.iter().map(|(k, v)| (*k, v.clone())));

    #[cfg(feature = "dktk")]
    map.extend(dktk::SAMPLE_TYPE_WORKAROUNDS.iter().map(|(k, v)| (*k, v.clone())));

    map
});

// code lists needed depending on the criteria selected
pub static CRITERION_CODE_LISTS: Lazy<HashMap<&str, Vec<&str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature = "bbmri")]
    map.extend(bbmri::CRITERION_CODE_LISTS.iter().map(|(k, v)| (*k, v.clone())));

    #[cfg(feature = "dktk")]
    map.extend(dktk::CRITERION_CODE_LISTS.iter().map(|(k, v)| (*k, v.clone())));

    map
});

// CQL snippets depending on the criteria
pub static CQL_SNIPPETS: Lazy<HashMap<(&str, CriterionRole), &str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    #[cfg(feature = "bbmri")]
    map.extend(bbmri::CQL_SNIPPETS.iter().map(|(k, v)| (*k, *v)));

    #[cfg(feature = "dktk")]
    map.extend(dktk::CQL_SNIPPETS.iter().map(|(k, v)| (*k, *v)));

    map
});
pub static MANDATORY_CODE_SYSTEMS: Lazy<IndexSet<&str>> = Lazy::new(|| {
    let mut set = IndexSet::new();

    #[cfg(feature = "bbmri")]
    set.extend(bbmri::MANDATORY_CODE_LISTS.iter().copied());

    #[cfg(feature = "dktk")]
    set.extend(dktk::MANDATORY_CODE_LISTS.iter().copied());

    set
});

pub static CQL_TEMPLATE: Lazy<&'static str> = Lazy::new(|| {
    #[cfg(all(feature = "bbmri", not(feature = "dktk")))]
    return *bbmri::CQL_TEMPLATE;
    
    #[cfg(all(feature = "dktk", not(feature = "bbmri")))]
    return *dktk::CQL_TEMPLATE;
    
    #[cfg(all(feature = "bbmri", feature = "dktk"))]
    {
        let mut template = String::new();
        template.push_str(*bbmri::CQL_TEMPLATE);
        template.push_str(*dktk::CQL_TEMPLATE);
        template.leak()
    }
    
    #[cfg(not(any(feature = "bbmri", feature = "dktk")))]
    ""
});

pub static BODY: Lazy<&'static str> = Lazy::new(|| {
    #[cfg(all(feature = "bbmri", not(feature = "dktk")))]
    return *bbmri::BODY;
    
    #[cfg(all(feature = "dktk", not(feature = "bbmri")))]
    return *dktk::BODY;
    
    #[cfg(all(feature = "bbmri", feature = "dktk"))]
    {
        let mut body = String::new();
        body.push_str(*bbmri::BODY);
        body.push_str(*dktk::BODY);
        body.leak()
    }
    
    #[cfg(not(any(feature = "bbmri", feature = "dktk")))]
    ""
});
