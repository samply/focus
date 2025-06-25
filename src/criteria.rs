use std::collections::BTreeMap;

pub type Criteria = BTreeMap<String, u64>; //stratifier

pub type Stratifiers = BTreeMap<String, Criteria>; //group

#[allow(dead_code)]
pub type CriteriaGroups = BTreeMap<String, Stratifiers>; //the entire structure containing groups

// groups of groups of criteria follow the measure report structure
// for example criteria "female", "male", "other", and "unknown" belong to the group "gender", and the group "gender" together with the group "age" belongs to the group of groups "patient"
// 2025-06-06 refactored extraction to remove groups and add all the stratifiers into one BTreeMap, function preserved in case another project needs groups

fn combine_maps(map1: Criteria, map2: Criteria) -> Criteria { // here individual criteria are combined and their numbers added, for example 2 maps of gender criteria (see test)
    let mut combined_map = map1;
    for (key, value) in map2 {
        *combined_map.entry(key).or_insert(0) += value;
    }
    combined_map
}

pub fn combine_criteria_groups(group1: Stratifiers, group2: Stratifiers) -> Stratifiers { // here criteria groups are combined in a way that criteria maps having the same key are combined, for example 2 maps of patients
    let mut combined_group = group1;

    for (key, criteria) in group2 {
        let maybe_criteria = combined_group.get(&key);
        match maybe_criteria {
            Some(existing_criteria) => {
                combined_group.insert(key, combine_maps(existing_criteria.clone(), criteria));
            }
            None => {
                combined_group.insert(key, criteria);
            }
        }
    }
    combined_group
}

#[allow(dead_code)]
fn combine_groups_of_criteria_groups(groups1: CriteriaGroups, groups2: CriteriaGroups) -> CriteriaGroups { // here groups of criteria groups are combined in a way that groups of criteria groups are combined using the previous function
    let mut combined_groups = groups1; // this function is used to combine maps for all the sites

    for (key, criteria_group) in groups2 {
        let maybe_criteria_group = combined_groups.get(&key);
        match maybe_criteria_group {
            Some(existing_criteria_group) => {
                combined_groups.insert(key, combine_criteria_groups(existing_criteria_group.clone(), criteria_group));
            }
            None => {
                combined_groups.insert(key, criteria_group);
            }
        }
    }
    combined_groups
}


#[cfg(test)]
mod test {
    use super::*;

    const CRITERIA_GROUP_JSON: &str = r#"{"gender":{"female":20,"male":20,"other":10}}"#;
    const CRITERIA_GROUPS_JSON: &str = r#"{"patients":{"gender":{"female":20,"male":20,"other":10}}}"#;

    #[test]
    fn test_combining_criteria_groups_serialization() {
        let map1: Criteria = [("male".into(), 20), ("female".into(), 10)]
            .iter()
            .cloned()
            .collect();

        let map2: Criteria = [("female".into(), 10), ("other".into(), 10)]
            .iter()
            .cloned()
            .collect();

        let combined_map = combine_maps(map1.clone(), map2.clone());

        let criteria_group: Stratifiers =
            [("gender".into(), combined_map)].iter().cloned().collect();

        let criteria_group_json =
            serde_json::to_string(&criteria_group).expect("Failed to serialize JSON");

        pretty_assertions::assert_eq!(CRITERIA_GROUP_JSON, criteria_group_json);

        let criteria_group1: Stratifiers =
        [("gender".into(), map1)].iter().cloned().collect();

        let criteria_group2: Stratifiers =
        [("gender".into(), map2)].iter().cloned().collect();

        let criteria_group_combined = combine_criteria_groups(criteria_group1.clone(), criteria_group2.clone());

        let criteria_group_combined_json =  serde_json::to_string(&criteria_group_combined).expect("Failed to serialize JSON");
        
        pretty_assertions::assert_eq!(CRITERIA_GROUP_JSON, criteria_group_combined_json);

        let criteria_groups1 : CriteriaGroups = 
        [("patients".into(), criteria_group1)].iter().cloned().collect();

        let criteria_groups2 : CriteriaGroups = 
        [("patients".into(), criteria_group2)].iter().cloned().collect();

        let criteria_groups_combined : CriteriaGroups = combine_groups_of_criteria_groups(criteria_groups1, criteria_groups2);

        let criteria_groups_combined_json =  serde_json::to_string(&criteria_groups_combined).expect("Failed to serialize JSON");

        pretty_assertions::assert_eq!(CRITERIA_GROUPS_JSON, criteria_groups_combined_json);


    }
}
