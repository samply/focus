// TODO: put miabis stuff here

use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

pub static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> =     
    LazyLock::new(|| {
        HashMap::new() 
    });

pub static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> =
        LazyLock::new(|| {
        HashMap::new() 
    });

pub static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
        LazyLock::new(|| {
        HashMap::new() 
    });

pub static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> =
        LazyLock::new(|| {
        HashMap::new() 
    });

pub static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> =
        LazyLock::new(|| {
        IndexSet::new() 
    });

pub static CODE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> = // both sample type and storage temperature replacement codes
        LazyLock::new(|| {
        HashMap::new() 
    });
