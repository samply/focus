use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub fields: BTreeMap<String, Field>,
    pub scopes: BTreeMap<String, Scope>,
    pub counts: BTreeMap<String, Count>,
    pub stratifiers: BTreeMap<String, Stratifier>,
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
    serde_yaml_ng::from_str(yaml).map_err(|e| e.to_string())
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
