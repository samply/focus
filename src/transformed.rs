use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

pub type Facets = BTreeMap<String, u64>; //stratifier

pub type Totals = BTreeMap<String, u64>;

pub type Stratifiers = BTreeMap<String, Facets>; //group or a collection of all stratifiers on the same level

#[derive(Debug, Deserialize, Serialize)]
pub struct Transformed {
    pub stratifiers: Stratifiers,
    pub totals: Totals,
}

#[allow(dead_code)]
pub type StratifierGroups = BTreeMap<String, Stratifiers>; //the entire structure containing groups

// initially groups of stratifiers followed the measure report structure
// for example facets "female", "male", "other", and "unknown" belonged to the stratifier "gender", and the stratifier "gender" together with the stratifier "age" belonged to the group of stratifiers "patient"
// 2025-06-06 refactored extraction to remove groups and add all the stratifiers into one BTreeMap, function preserved in case another project needs groups

fn combine_maps(map1: Facets, map2: Facets) -> Facets {
    // here individual facets are combined and their numbers added, for example 2 maps of gender facets (see test)
    let mut combined_map = map1;
    for (key, value) in map2 {
        *combined_map.entry(key).or_insert(0) += value;
    }
    combined_map
}

pub fn combine_stratifier_groups(group1: Stratifiers, group2: Stratifiers) -> Stratifiers {
    // here stratifier groups are combined in a way that groups having the same key are combined, for example 2 maps of patients
    let mut combined_group = group1;

    for (key, facets) in group2 {
        let maybe_facets = combined_group.get(&key);
        match maybe_facets {
            Some(existing_facets) => {
                combined_group.insert(key, combine_maps(existing_facets.clone(), facets));
            }
            None => {
                combined_group.insert(key, facets);
            }
        }
    }
    combined_group
}

#[allow(dead_code)]
fn combine_groups_of_stratifiers(groups1: StratifierGroups, groups2: StratifierGroups) -> StratifierGroups {
    // here groups of stratifiers are combined using the previous function
    let mut combined_groups = groups1; // this function is used to combine maps for all the sites

    for (key, stratifier_group) in groups2 {
        let maybe_stratifier_group = combined_groups.get(&key);
        match maybe_stratifier_group {
            Some(existing_stratifier_group) => {
                combined_groups.insert(
                    key,
                    combine_stratifier_groups(existing_stratifier_group.clone(), stratifier_group),
                );
            }
            None => {
                combined_groups.insert(key, stratifier_group);
            }
        }
    }
    combined_groups
}

#[cfg(test)]
mod test {
    use super::*;

    const STRATIFIER_JSON: &str = r#"{"gender":{"female":20,"male":20,"other":10}}"#;
    const STRATIFIER_GROUP_JSON: &str =
        r#"{"patients":{"gender":{"female":20,"male":20,"other":10}}}"#;

    #[test]
    fn test_combining_stratifiers_serialization() {
        let map1: Facets = [("male".into(), 20), ("female".into(), 10)]
            .iter()
            .cloned()
            .collect();

        let map2: Facets = [("female".into(), 10), ("other".into(), 10)]
            .iter()
            .cloned()
            .collect();

        let combined_map = combine_maps(map1.clone(), map2.clone());

        let stratifiers: Stratifiers = [("gender".into(), combined_map)].iter().cloned().collect();

        let stratifiers_json =
            serde_json::to_string(&stratifiers).expect("Failed to serialize JSON");

        pretty_assertions::assert_eq!(STRATIFIER_JSON, stratifiers_json);

        let stratifier_group1: Stratifiers = [("gender".into(), map1)].iter().cloned().collect();

        let stratifier_group2: Stratifiers = [("gender".into(), map2)].iter().cloned().collect();

        let stratifier_group_combined =
            combine_stratifier_groups(stratifier_group1.clone(), stratifier_group2.clone());

        let stratifier_group_combined_json =
            serde_json::to_string(&stratifier_group_combined).expect("Failed to serialize JSON");

        pretty_assertions::assert_eq!(STRATIFIER_JSON, stratifier_group_combined_json);

        let stratifier_groups1: StratifierGroups = [("patients".into(), stratifier_group1)]
            .iter()
            .cloned()
            .collect();

        let stratifier_groups2: StratifierGroups = [("patients".into(), stratifier_group2)]
            .iter()
            .cloned()
            .collect();

        let stratifier_groups_combined: StratifierGroups =
            combine_groups_of_stratifiers(stratifier_groups1, stratifier_groups2);

        let stratifier_groups_combined_json =
            serde_json::to_string(&stratifier_groups_combined).expect("Failed to serialize JSON");

        pretty_assertions::assert_eq!(STRATIFIER_GROUP_JSON, stratifier_groups_combined_json);
    }
}
