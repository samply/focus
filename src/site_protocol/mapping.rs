use std::collections::BTreeMap;

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

pub fn load(project: &str) -> Result<Mapping, String> {
    let yaml = match project {
        "dktk" => include_str!("mappings/dktk.yaml"),
        _ => return Err(format!("unknown project \"{project}\"")),
    };
    let mapping: Mapping = serde_saphyr::from_str(yaml).map_err(|e| e.to_string())?;
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
        for project in ["dktk"] {
            super::load(project).unwrap();
        }
    }
}
