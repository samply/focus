use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub fields: BTreeMap<String, Field>,
    pub scopes: BTreeMap<String, Scope>,
    pub populations: BTreeMap<String, Population>,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    pub resource_type: String,
    pub retrieve: Option<String>,
    pub temporal_date: Option<String>,
    pub fields: BTreeMap<String, Field>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Population {
    pub resource_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Stratifier {
    pub population: String,
    pub cql: Option<String>,
    pub buckets: Option<Vec<Bucket>>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Bucket {
    pub key: String,
    pub cql: String,
}

impl Scope {
    pub fn retrieve(&self) -> String {
        self.retrieve
            .clone()
            .unwrap_or_else(|| format!("[{}]", self.resource_type))
    }
}

pub fn load(project: &str) -> Result<Mapping, String> {
    let yaml = match project {
        "dktk" => include_str!("mappings/dktk.yaml"),
        _ => return Err(format!("unknown project \"{project}\"")),
    };
    let mapping: Mapping = serde_yaml_ng::from_str(yaml).map_err(|e| e.to_string())?;
    mapping.check()?;
    Ok(mapping)
}

impl Mapping {
    fn check(&self) -> Result<(), String> {
        let patients = self
            .populations
            .values()
            .filter(|p| p.resource_type == "Patient")
            .count();
        if patients != 1 {
            return Err("exactly one population must have resourceType Patient".into());
        }
        for (key, s) in &self.stratifiers {
            if !self.populations.contains_key(&s.population) {
                return Err(format!(
                    "stratifier \"{key}\" has unknown population \"{}\"",
                    s.population
                ));
            }
            if s.cql.is_some() == s.buckets.is_some() {
                return Err(format!(
                    "stratifier \"{key}\" needs exactly one of cql and buckets"
                ));
            }
        }
        Ok(())
    }

    pub fn patient_population(&self) -> &str {
        self.populations
            .iter()
            .find(|(_, p)| p.resource_type == "Patient")
            .map(|(key, _)| key.as_str())
            .unwrap_or_default()
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
