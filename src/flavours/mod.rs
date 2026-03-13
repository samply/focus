use std::{collections::HashMap, hash::Hash, str::FromStr};

use indexmap::IndexSet;

use crate::errors::FocusError;

mod bbmri;
mod cce;
mod dhki;
mod dktk;
mod itcc;
mod miabis;
mod nngm;
mod pscc;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub enum CriterionRole {
    Query,
    Filter,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Flavour {
    //not 1:1 with project anymore
    Bbmri,
    Dktk,
    Cce,
    Dhki,
    Nngm,
    Itcc,
    Pscc,
    Miabis,
}

impl FromStr for Flavour {
    type Err = FocusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bbmri" => Ok(Flavour::Bbmri),
            "dktk" => Ok(Flavour::Dktk),
            "cce" => Ok(Flavour::Cce),
            "nngm" => Ok(Flavour::Nngm),
            "dhki" => Ok(Flavour::Dhki),
            "itcc" => Ok(Flavour::Itcc),
            "pscc" => Ok(Flavour::Pscc),
            "miabis" => Ok(Flavour::Miabis),
            _ => Err(FocusError::UnknownProject(s.to_string())),
        }
    }
}

impl Flavour {
    pub fn get_code_lists(&self) -> &'static HashMap<&'static str, &'static str> {
        match self {
            Flavour::Bbmri => &bbmri::CODE_LISTS,
            Flavour::Dktk => &dktk::CODE_LISTS,
            Flavour::Cce => &cce::CODE_LISTS,
            Flavour::Dhki => &dhki::CODE_LISTS,
            Flavour::Nngm => &nngm::CODE_LISTS,
            Flavour::Itcc => &itcc::CODE_LISTS,
            Flavour::Pscc => &pscc::CODE_LISTS,
            Flavour::Miabis => &miabis::CODE_LISTS,
        }
    }

    pub fn get_observation_loinc_codes(&self) -> &'static HashMap<&'static str, &'static str> {
        match self {
            Flavour::Bbmri => &bbmri::OBSERVATION_LOINC_CODES,
            Flavour::Dktk => &dktk::OBSERVATION_LOINC_CODES,
            Flavour::Cce => &cce::OBSERVATION_LOINC_CODES,
            Flavour::Dhki => &dhki::OBSERVATION_LOINC_CODES,
            Flavour::Nngm => &nngm::OBSERVATION_LOINC_CODES,
            Flavour::Itcc => &itcc::OBSERVATION_LOINC_CODES,
            Flavour::Pscc => &pscc::OBSERVATION_LOINC_CODES,
            Flavour::Miabis => &miabis::OBSERVATION_LOINC_CODES,
        }
    }

    pub fn get_code_workarounds(&self) -> &'static HashMap<&'static str, Vec<&'static str>> {
        // used for all code workarounds, different criteria do not contain overlapping codes
        match self {
            Flavour::Bbmri => &bbmri::CODE_WORKAROUNDS,
            Flavour::Dktk => &dktk::CODE_WORKAROUNDS,
            Flavour::Cce => &cce::CODE_WORKAROUNDS,
            Flavour::Dhki => &dhki::CODE_WORKAROUNDS,
            Flavour::Nngm => &nngm::CODE_WORKAROUNDS,
            Flavour::Itcc => &itcc::CODE_WORKAROUNDS,
            Flavour::Pscc => &pscc::CODE_WORKAROUNDS,
            Flavour::Miabis => &miabis::CODE_WORKAROUNDS,
        }
    }

    pub fn get_criterion_code_lists(&self) -> &'static HashMap<&'static str, Vec<&'static str>> {
        match self {
            Flavour::Bbmri => &bbmri::CRITERION_CODE_LISTS,
            Flavour::Dktk => &dktk::CRITERION_CODE_LISTS,
            Flavour::Cce => &cce::CRITERION_CODE_LISTS,
            Flavour::Dhki => &dhki::CRITERION_CODE_LISTS,
            Flavour::Nngm => &nngm::CRITERION_CODE_LISTS,
            Flavour::Itcc => &itcc::CRITERION_CODE_LISTS,
            Flavour::Pscc => &pscc::CRITERION_CODE_LISTS,
            Flavour::Miabis => &miabis::CRITERION_CODE_LISTS,
        }
    }

    pub fn get_cql_snippets(
        &self,
    ) -> &'static HashMap<(&'static str, CriterionRole), &'static str> {
        match self {
            Flavour::Bbmri => &bbmri::CQL_SNIPPETS,
            Flavour::Dktk => &dktk::CQL_SNIPPETS,
            Flavour::Cce => &cce::CQL_SNIPPETS,
            Flavour::Dhki => &dhki::CQL_SNIPPETS,
            Flavour::Nngm => &nngm::CQL_SNIPPETS,
            Flavour::Itcc => &itcc::CQL_SNIPPETS,
            Flavour::Pscc => &pscc::CQL_SNIPPETS,
            Flavour::Miabis => &miabis::CQL_SNIPPETS,
        }
    }

    pub fn get_mandatory_code_lists(&self) -> &'static IndexSet<&'static str> {
        match self {
            Flavour::Bbmri => &bbmri::MANDATORY_CODE_LISTS,
            Flavour::Dktk => &dktk::MANDATORY_CODE_LISTS,
            Flavour::Cce => &cce::MANDATORY_CODE_LISTS,
            Flavour::Dhki => &dhki::MANDATORY_CODE_LISTS,
            Flavour::Nngm => &nngm::MANDATORY_CODE_LISTS,
            Flavour::Itcc => &itcc::MANDATORY_CODE_LISTS,
            Flavour::Pscc => &pscc::MANDATORY_CODE_LISTS,
            Flavour::Miabis => &miabis::MANDATORY_CODE_LISTS,
        }
    }

    pub fn get_cql_template(&self) -> &'static str {
        match self {
            Flavour::Bbmri => include_str!("bbmri/template.cql"),
            Flavour::Dktk => include_str!("dktk/template.cql"),
            Flavour::Cce => include_str!("cce/template.cql"),
            Flavour::Dhki => include_str!("dhki/template.cql"),
            Flavour::Nngm => include_str!("nngm/template.cql"),
            Flavour::Itcc => include_str!("itcc/template.cql"),
            Flavour::Pscc => include_str!("pscc/template.cql"),
            Flavour::Miabis => include_str!("miabis/template.cql"),
        }
    }

    pub fn get_body(&self) -> &'static str {
        match self {
            Flavour::Bbmri => include_str!("bbmri/body.json"),
            Flavour::Dktk => include_str!("dktk/body.json"),
            Flavour::Cce => include_str!("cce/body.json"),
            Flavour::Dhki => include_str!("dhki/body.json"),
            Flavour::Nngm => include_str!("nngm/body.json"),
            Flavour::Itcc => include_str!("itcc/body.json"),
            Flavour::Pscc => include_str!("pscc/body.json"),
            Flavour::Miabis => include_str!("miabis/body.json"),
        }
    }
}
