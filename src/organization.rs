use std::collections::HashMap;

use crate::{
    blaze::get_organization_directory_id, errors::FocusError, mr::MeasureReport, mr::Stratifier,
};
use tracing::trace;

type OrganizationFhirId = String;
type OrganizationDirectoryId = String;
pub struct OrganizationCache {
    pub cache: HashMap<OrganizationFhirId, OrganizationDirectoryId>,
} // the cache does not expire, directory IDs are going to change much less often than the BH restarts (if ever)

pub async fn replace_org_fhir_ids_with_org_directory_ids(
    mut result_mr: MeasureReport,
    organization_cache: &mut OrganizationCache,
) -> Result<MeasureReport, FocusError> {
    for group in &mut result_mr.group.iter_mut() {
        match &group.code.text[..] {
            "patient" | "patients" => {
                for stratifier in group.stratifier.iter_mut() {
                    let mut is_custodian = false;
                    for code in &stratifier.code {
                        if &code.text[..] == "Custodian" {
                            is_custodian = true;
                        }
                    }
                    if is_custodian {
                        process_strata(stratifier, organization_cache).await;
                    }
                }
            }
            "specimen" | "specimens" => {
                for stratifier in group.stratifier.iter_mut() {
                    let mut is_custodian = false;
                    for code in &stratifier.code {
                        if &code.text[..] == "Custodian-specimen" {
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

    Ok(result_mr)
}

async fn process_strata(stratifier: &mut Stratifier, organization_cache: &mut OrganizationCache) {
    for strata in stratifier.stratum.iter_mut() {
        for stratum in strata.iter_mut() {
            let text = &stratum.value.text[..];

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
                        directory_id
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
                        &(org_id.clone())
                    }
                } else {
                    //Organization FHIR ID is in wrong format
                    &("null".to_string())
                };
                stratum.value.text = directory_id.clone().to_string();
            }
        }
    }
}
