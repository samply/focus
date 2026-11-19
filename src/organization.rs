use std::collections::HashMap;

use crate::{blaze::get_organization_directory_id, errors::FocusError};
use serde_json::Value;
use tracing::trace;

type OrganizationFhirId = String;
type OrganizationDirectoryId = String;
pub struct OrganizationCache {
    pub cache: HashMap<OrganizationFhirId, OrganizationDirectoryId>,
} // the cache does not expire, directory IDs are going to change much less often than the BH restarts (if ever)

/// Replaces Organization FHIR IDs in the Custodian stratifiers with Organization directory IDs.
///
/// Works on plain JSON on purpose: MeasureReports of other projects (e.g. EXLIQUID, whose
/// groups and stratifiers only have a `coding` and whose strata have `component`s instead of
/// a `value`) must pass through unchanged instead of failing to parse or losing fields.
pub async fn replace_org_fhir_ids_with_org_directory_ids(
    mut result_measure_report: Value,
    organization_cache: &mut OrganizationCache,
) -> Result<Value, FocusError> {
    let Some(groups) = result_measure_report
        .get_mut("group")
        .and_then(Value::as_array_mut)
    else {
        return Ok(result_measure_report);
    };
    for group in groups {
        match group.pointer("/code/text").and_then(Value::as_str) {
            Some("patient" | "patients") => {
                for stratifier in stratifiers_mut(group) {
                    let mut is_custodian = false;
                    for code in stratifier["code"].as_array().into_iter().flatten() {
                        if code["text"] == "Custodian" {
                            is_custodian = true;
                        }
                    }
                    if is_custodian {
                        process_strata(stratifier, organization_cache).await;
                    }
                }
            }
            Some("specimen" | "specimens") => {
                for stratifier in stratifiers_mut(group) {
                    let mut is_custodian = false;
                    for code in stratifier["code"].as_array().into_iter().flatten() {
                        if code["text"] == "Custodian-specimen" {
                            is_custodian = true;
                        }
                    }
                    if is_custodian {
                        process_strata(stratifier, organization_cache).await;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(result_measure_report)
}

/// The group's stratifiers, or none if the group has no stratifier array.
fn stratifiers_mut(group: &mut Value) -> impl Iterator<Item = &mut Value> {
    group
        .get_mut("stratifier")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
}

async fn process_strata(stratifier: &mut Value, organization_cache: &mut OrganizationCache) {
    let Some(strata) = stratifier.get_mut("stratum").and_then(Value::as_array_mut) else {
        return;
    };
    for stratum in strata {
        let Some(text) = stratum
            .pointer("/value/text")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };
        if text.contains("Organization") {
            let fhir_id_maybe = text.split_once("Organization/");
            let directory_id = if let Some((_, fhir_id)) = fhir_id_maybe {
                trace!("Checking organization cache for FHIR ID {}", &fhir_id);
                let cached = organization_cache.cache.get(fhir_id);
                if let Some(directory_id) = cached {
                    trace!(
                        "Found in cache: FHIR ID: {}, Directory ID: {}",
                        fhir_id,
                        directory_id
                    );
                    directory_id.as_str()
                } else {
                    let org_id = get_organization_directory_id(fhir_id.to_string())
                        .await
                        .unwrap_or("null".to_string());
                    // the result should still be returned, in case of "null", the default collection of the biobank is sent to the Negotiator
                    organization_cache
                        .cache
                        .insert(fhir_id.to_string().clone(), org_id.clone());
                    trace!(
                        "Got from Blaze: FHIR ID: {}, Directory ID: {}",
                        fhir_id,
                        org_id
                    );
                    &(org_id.to_owned())
                }
            } else {
                //Organization FHIR ID is in wrong format
                "null"
            };
            stratum["value"]["text"] = Value::String(directory_id.to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_MEASURE_REPORT_EXLIQUID_COMPONENTS: &str =
        include_str!("../resources/test/measure_report_exliquid_components.json");

    fn empty_cache() -> OrganizationCache {
        OrganizationCache {
            cache: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_exliquid_measure_report() {
        // EXLIQUID groups and stratifiers only have a `coding`, and the specimen strata have
        // `component`s instead of a `value`.
        let measure_report: Value =
            serde_json::from_str(EXAMPLE_MEASURE_REPORT_EXLIQUID_COMPONENTS).unwrap();

        let result =
            replace_org_fhir_ids_with_org_directory_ids(measure_report.clone(), &mut empty_cache())
                .await
                .expect("EXLIQUID MeasureReport should be accepted");

        pretty_assertions::assert_eq!(measure_report, result);
    }
}
