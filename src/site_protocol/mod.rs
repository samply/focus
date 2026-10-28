// Site protocol queries: an AST in, counts out. Used by lens3 explorers.
// What a project may query lives in mappings/<project>.yaml.

mod ast;
mod cql;
mod mapping;

use tracing::warn;

use crate::beam::beam_result;
use crate::blaze;
use crate::config::{EndpointType, Obfuscate, CONFIG};
use crate::errors::FocusError;
use crate::{BeamResult, BeamTask};

#[derive(Debug)]
pub enum Error {
    /// The query itself is wrong. The message is sent back.
    Rejected(String),
    /// Something broke while running it. Only logged, as it might contain
    /// data from the store.
    Failed(String),
}

impl From<FocusError> for Error {
    fn from(e: FocusError) -> Self {
        Error::Failed(e.to_string())
    }
}

pub async fn run(task: &BeamTask, project: &str, version: u32) -> BeamResult {
    let from = CONFIG.beam_app_id_long.clone();
    let to = vec![task.from.clone()];
    match handle(&task.body, project, version).await {
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

async fn handle(body: &str, project: &str, version: u32) -> Result<String, Error> {
    if version != 1 {
        return Err(Error::Rejected(format!(
            "unsupported site_protocol {version}"
        )));
    }
    // TODO: obfuscate
    if CONFIG.obfuscate == Obfuscate::Yes && !CONFIG.unobfuscated.iter().any(|p| p == project) {
        return Err(Error::Rejected(
            "site protocol results can't be obfuscated yet, set OBFUSCATE=no".into(),
        ));
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
    let report = blaze::run_cql_query(&library, &measure).await?;
    let response = cql::read_report(&report)?;
    serde_json::to_string(&response).map_err(|e| Error::Failed(e.to_string()))
}
