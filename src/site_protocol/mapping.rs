use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub fields: BTreeMap<String, Field>,
    pub scopes: BTreeMap<String, Scope>,
    pub counts: BTreeMap<String, Count>,
    pub stratifiers: BTreeMap<String, Stratifier>,
    /// How much one patient can change a count of this resource type, for
    /// obfuscation. Patient is always 1.
    #[serde(default)]
    pub sensitivity: BTreeMap<String, f64>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(rename = "type")]
    pub type_: FieldType,
    pub cql: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Values,
    Date,
    Range,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub resource_type: String,
    pub filter: Option<String>,
    pub temporal_date: Option<String>,
    pub fields: BTreeMap<String, Field>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Count {
    pub resource_type: String,
    pub filter: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stratifier {
    pub resource_type: String,
    pub filter: Option<String>,
    pub cql: String,
}

static MAPPINGS: LazyLock<HashMap<&'static str, Mapping>> = LazyLock::new(|| {
    [("dktk", include_str!("mappings/dktk.yaml"))]
        .into_iter()
        .map(|(project, yaml)| {
            let mapping = parse(yaml).unwrap_or_else(|e| panic!("mapping \"{project}\": {e}"));
            (project, mapping)
        })
        .collect()
});

pub fn get(project: &str) -> Result<&'static Mapping, String> {
    MAPPINGS
        .get(project)
        .ok_or_else(|| format!("unknown project \"{project}\""))
}

fn parse(yaml: &str) -> Result<Mapping, String> {
    let mapping: Mapping = serde_saphyr::from_str(yaml).map_err(|e| e.to_string())?;
    // Check that all resource types used by counts and stratifiers have a
    // sensitivity. Patient defaults to 1 but can be overridden in the mapping.
    let resource_types = mapping.counts.values().map(|c| &c.resource_type);
    let resource_types =
        resource_types.chain(mapping.stratifiers.values().map(|s| &s.resource_type));
    for resource_type in resource_types {
        if resource_type != "Patient" && !mapping.sensitivity.contains_key(resource_type) {
            return Err(format!("no sensitivity for {resource_type}"));
        }
    }
    Ok(mapping)
}

impl Mapping {
    pub fn sensitivity(&self, resource_type: &str) -> f64 {
        self.sensitivity.get(resource_type).copied().unwrap_or(1.)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn mappings_load() {
        std::sync::LazyLock::force(&super::MAPPINGS);
    }
}
