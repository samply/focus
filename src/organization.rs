use std::collections::HashMap;

use crate::{
    errors::FocusError,
    blaze::get_organization_directory_id,
    mr::MeasureReport
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

type OrganizationFhirId = String;
type OrganizationDirectoryId = String;
pub struct OrganizationCache {
    pub cache: HashMap<OrganizationFhirId, OrganizationDirectoryId>,
} // the cache does not expire, directory IDs are going to change much less often than the BH restarts (if ever)

pub fn replace_org_fhir_ids_with_org_directory_ids(cql_result_new: &String, organization_cache :&mut OrganizationCache) -> String {
    cql_result_new.clone()
}


#[cfg(test)]
mod test {

    use super::*;


}