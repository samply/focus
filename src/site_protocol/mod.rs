// Site protocol queries: an AST in, counts out. Used by lens3 explorers.
// What a project may query lives in mappings/<project>.yaml.

mod ast;
mod cql;
mod mapping;

use std::sync::Arc;

use laplace_rs::{get_from_cache_or_privatize, ObfCache, ObfuscateBelow10Mode};
use tokio::sync::Mutex;
use tracing::warn;

use crate::beam::beam_result;
use crate::blaze;
use crate::config::{EndpointType, Obfuscate, CONFIG};
use crate::{BeamResult, BeamTask};

#[derive(Debug)]
pub enum Error {
    /// The query itself is wrong. The message is sent back.
    Rejected(String),
    /// Something broke while running it. Only logged, as it might contain
    /// data from the store.
    Failed(String),
}

pub async fn run(
    task: &BeamTask,
    project: &str,
    version: u32,
    obf_cache: Arc<Mutex<ObfCache>>,
) -> BeamResult {
    let from = CONFIG.beam_app_id_long.clone();
    let to = vec![task.from.clone()];
    match handle(&task.body, project, version, obf_cache).await {
        Ok(body) => beam_result::succeeded(from, to, task.id, body),
        Err(Error::Rejected(message)) => {
            warn!("Rejected site protocol query: {message}");
            beam_result::perm_failed(from, to, task.id, message)
        }
        Err(Error::Failed(message)) => {
            warn!("Failed to execute site protocol query: {message}");
            beam_result::perm_failed(from, to, task.id, "Failed to execute query.".into())
        }
    }
}

async fn handle(
    body: &str,
    project: &str,
    version: u32,
    obf_cache: Arc<Mutex<ObfCache>>,
) -> Result<String, Error> {
    if version != 1 {
        return Err(Error::Rejected(format!(
            "unsupported site_protocol {version}"
        )));
    }
    if !matches!(
        CONFIG.endpoint_type,
        EndpointType::Blaze | EndpointType::BlazeAndSql
    ) {
        return Err(Error::Rejected(
            "site protocol queries need a Blaze endpoint".into(),
        ));
    }

    let mapping = mapping::load(project).map_err(Error::Rejected)?;
    let query: ast::Query =
        serde_json::from_str(body).map_err(|e| Error::Rejected(format!("invalid query: {e}")))?;
    let (library, measure) = cql::generate(&query.expression, &mapping)?;
    let report = blaze::run_cql_query(&library, &measure)
        .await
        .map_err(|e| Error::Failed(e.to_string()))?;
    let mut response = cql::read_report(&report)?;
    if CONFIG.obfuscate == Obfuscate::Yes && !CONFIG.unobfuscated.iter().any(|p| p == project) {
        obfuscate(&mut response, &mapping, &mut *obf_cache.lock().await)?;
    }
    serde_json::to_string(&response).map_err(|e| Error::Failed(e.to_string()))
}

/// Laplace noise on every count and stratum, scaled by the sensitivity of what
/// is counted. Counts and strata are cached apart (bins 1 and 2), as in the
/// other queries.
fn obfuscate(
    response: &mut cql::Response,
    mapping: &mapping::Mapping,
    cache: &mut ObfCache,
) -> Result<(), Error> {
    let mut rng = rand::rng();
    let mut privatize = |value: &mut u64, sensitivity: f64, bin: usize| {
        let below_10 = match CONFIG.obfuscate_below_10_mode {
            0 => ObfuscateBelow10Mode::Zero,
            1 => ObfuscateBelow10Mode::Ten,
            _ => ObfuscateBelow10Mode::Obfuscate,
        };
        *value = get_from_cache_or_privatize(
            *value,
            sensitivity,
            CONFIG.epsilon,
            bin,
            Some(&mut *cache),
            CONFIG.obfuscate_zero,
            below_10,
            CONFIG.rounding_step,
            None,
            &mut rng,
        )
        .map_err(|e| Error::Failed(format!("obfuscation failed: {e}")))?;
        Ok::<_, Error>(())
    };
    for (key, value) in &mut response.counts {
        privatize(
            value,
            mapping.sensitivity(&mapping.counts[key].resource_type),
            1,
        )?;
    }
    for (key, strata) in &mut response.stratifiers {
        let sensitivity = mapping.sensitivity(&mapping.stratifiers[key].resource_type);
        for value in strata.values_mut() {
            privatize(value, sensitivity, 2)?;
        }
    }
    Ok(())
}
