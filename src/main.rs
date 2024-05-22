mod ast;
mod banner;
mod beam;
mod blaze;
mod config;
mod errors;
mod exporter;
mod graceful_shutdown;
mod logger;

mod intermediate_rep;
mod task_processing;
mod util;

use base64::engine::general_purpose;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use beam_lib::{TaskRequest, TaskResult};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use laplace_rs::ObfCache;
use tokio::sync::Mutex;

use crate::blaze::parse_blaze_query;
use crate::config::EndpointType;
use crate::util::{is_cql_tampered_with, obfuscate_counts_mr};
use crate::{config::CONFIG, errors::FocusError};
use blaze::CqlQuery;

use std::collections::HashMap;
use std::ops::DerefMut;
use std::process::ExitCode;
use std::str;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{process::exit, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

// result cache
type SearchQuery = String;
type Obfuscated = bool;
type Report = String;
type Created = std::time::SystemTime; //epoch
type BeamTask = TaskRequest<String>;
type BeamResult = TaskResult<beam_lib::RawString>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Metadata {
    project: String,
    task_type: Option<exporter::TaskType>,
}

#[derive(Debug, Clone, Default)]
struct ReportCache {
    cache: HashMap<(SearchQuery, Obfuscated), (Report, Created)>,
}

impl ReportCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        if let Some(filename) = CONFIG.queries_to_cache_file_path.clone() {
            let lines = util::read_lines(filename.clone().to_string());
            match lines {
                Ok(ok_lines) => {
                    for line in ok_lines {
                        let Ok(ok_line) = line else {
                            warn!("A line in the file {} is not readable", filename);
                            continue;
                        };
                        cache.insert((ok_line.clone(), false), ("".into(), UNIX_EPOCH));
                        cache.insert((ok_line, true), ("".into(), UNIX_EPOCH));
                    }
                }
                Err(_) => {
                    error!("The file {} cannot be opened", filename); //This shouldn't stop focus from running, it's just going to go to blaze every time, but that's not too slow
                }
            }
        }

        Self { cache }
    }
}

const REPORTCACHE_TTL: Duration = Duration::from_secs(86400); //24h

#[tokio::main]
pub async fn main() -> ExitCode {
    if let Err(e) = logger::init_logger() {
        error!("Cannot initalize logger: {}", e);
        exit(1);
    };
    banner::print_banner();

    let _ = CONFIG.api_key; // Initialize config

    tokio::select! {
        _ = graceful_shutdown::wait_for_signal() => {
            ExitCode::SUCCESS
        },
        code = main_loop() => {
            code
        }
    }
}

async fn main_loop() -> ExitCode {
    let endpoint_service_available: fn() -> BoxFuture<'static, bool> = match CONFIG.endpoint_type {
        EndpointType::Blaze => || blaze::check_availability().boxed(),
        EndpointType::Omop => || async { true }.boxed(), // TODO health check
    };
    let mut failures = 0;
    while !(beam::check_availability().await && endpoint_service_available().await) {
        failures += 1;
        if failures >= CONFIG.retry_count {
            error!(
                "Encountered too many errors -- exiting after {} attempts.",
                CONFIG.retry_count
            );
            return ExitCode::from(22);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        warn!(
            "Retrying connection (attempt {}/{})",
            failures,
            CONFIG.retry_count
        );
    };
    let report_cache = Arc::new(Mutex::new(ReportCache::new()));
    let obf_cache = Arc::new(Mutex::new(ObfCache {
        cache: Default::default(),
    }));
    task_processing::process_tasks(move |task| {
        let obf_cache = obf_cache.clone();
        let report_cache = report_cache.clone();
        process_task(task, obf_cache, report_cache).boxed_local()
    }).await;
    ExitCode::FAILURE
}

async fn process_task(
    task: &BeamTask,
    obf_cache: Arc<Mutex<ObfCache>>,
    report_cache: Arc<Mutex<ReportCache>>,
) -> Result<BeamResult, FocusError> {
    debug!("Processing task {}", task.id);

    let metadata: Metadata = serde_json::from_value(task.metadata.clone()).unwrap_or(Metadata {
        project: "default_obfuscation".to_string(),
        task_type: None
    });

    if metadata.project == "focus-healthcheck" {
        return Ok(beam::beam_result::succeeded(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            "healthy".into()
        ));
    }
    if metadata.project == "exporter" {
        if metadata.task_type.is_none() {
            return Err(FocusError::MissingExporterTaskType())
        }
        let body = &task.body;
        return Ok(run_exporter_query(task, body, metadata.task_type.unwrap()).await)?; //we already made sure that it is not None
    }

    if CONFIG.endpoint_type == EndpointType::Blaze {
        let query = parse_blaze_query(task)?;
        if query.lang == "cql" {
            // TODO: Change query.lang to an enum

            Ok(run_cql_query(task, &query, obf_cache, report_cache, metadata.project).await)?
        } else {
            warn!("Can't run queries with language {} in Blaze", query.lang);
            Ok(beam::beam_result::perm_failed(
                CONFIG.beam_app_id_long.clone(),
                vec![task.from.clone()],
                task.id,
                format!(
                    "Can't run queries with language {} and/or endpoint type {}",
                    query.lang, CONFIG.endpoint_type
                ),
            ))
        }
    } else if CONFIG.endpoint_type == EndpointType::Omop {
        let decoded = util::base64_decode(&task.body)?;
        let intermediate_rep_query: intermediate_rep::IntermediateRepQuery =
            serde_json::from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;
        //TODO check that the language is ast
        let query_decoded = general_purpose::STANDARD
            .decode(intermediate_rep_query.query)
            .map_err(FocusError::DecodeError)?;
        let ast: ast::Ast =
            serde_json::from_slice(&query_decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;

        Ok(run_intermediate_rep_query(task, ast).await)?
    } else {
        warn!(
            "Can't run queries with endpoint type {}",
            CONFIG.endpoint_type
        );
        Ok(beam::beam_result::perm_failed(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            format!(
                "Can't run queries with endpoint type {}",
                CONFIG.endpoint_type
            ),
        ))
    }
}

async fn run_cql_query(
    task: &BeamTask,
    query: &CqlQuery,
    obf_cache: Arc<Mutex<ObfCache>>,
    report_cache: Arc<Mutex<ReportCache>>,
    project: String,
) -> Result<BeamResult, FocusError> {
    let encoded_query =
        query.lib["content"][0]["data"]
            .as_str()
            .ok_or(FocusError::ParsingError(format!(
                "Not a valid library: Field .content[0].data not found. Library: {}",
                query.lib
            )))?;

    let mut key_exists = false;

    let obfuscate =
        CONFIG.obfuscate == config::Obfuscate::Yes && !CONFIG.unobfuscated.contains(&project);

    let report_from_cache = match report_cache
        .lock()
        .await
        .cache
        .get(&(encoded_query.to_string(), obfuscate))
    {
        Some(existing_report) => {
            key_exists = true;
            if SystemTime::now().duration_since(existing_report.1).unwrap() < REPORTCACHE_TTL {
                Some(existing_report.0.clone())
            } else {
                None
            }
        }
        None => None,
    };

    let cql_result_new = match report_from_cache {
        Some(some_report_from_cache) => some_report_from_cache.to_string(),
        None => {
            let query = replace_cql_library(query.clone())?;

            let cql_result = blaze::run_cql_query(&query.lib, &query.measure).await?;

            let cql_result_new: String = match obfuscate {
                true => obfuscate_counts_mr(
                    &cql_result,
                    obf_cache.lock().await.deref_mut(),
                    CONFIG.obfuscate_zero,
                    CONFIG.obfuscate_below_10_mode,
                    CONFIG.delta_patient,
                    CONFIG.delta_specimen,
                    CONFIG.delta_diagnosis,
                    CONFIG.delta_procedures,
                    CONFIG.delta_medication_statements,
                    CONFIG.delta_histo,
                    CONFIG.epsilon,
                    CONFIG.rounding_step,
                )?,
                false => cql_result,
            };

            if key_exists {
                report_cache.lock().await.cache.insert(
                    (encoded_query.to_string(), obfuscate),
                    (cql_result_new.clone(), std::time::SystemTime::now()),
                );
            }
            cql_result_new
        }
    };

    let result = beam_result(task.to_owned(), cql_result_new).unwrap_or_else(|e| {
        beam::beam_result::perm_failed(
            CONFIG.beam_app_id_long.clone(),
            vec![task.to_owned().from],
            task.to_owned().id,
            e.to_string(),
        )
    });

    Ok(result)
}

async fn run_intermediate_rep_query(
    task: &BeamTask,
    ast: ast::Ast,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let mut intermediate_rep_result = intermediate_rep::post_ast(ast).await?;

    if let Some(provider_icon) = CONFIG.provider_icon.clone() {
        intermediate_rep_result = intermediate_rep_result.replacen(
            '{',
            format!(r#"{{"provider_icon":"{}","#, provider_icon).as_str(),
            1,
        );
    }

    if let Some(provider) = CONFIG.provider.clone() {
        intermediate_rep_result = intermediate_rep_result.replacen(
            '{',
            format!(r#"{{"provider":"{}","#, provider).as_str(),
            1,
        );
    }

    let result = beam_result(task.to_owned(), intermediate_rep_result).unwrap_or_else(|e| {
        err.body = beam_lib::RawString(e.to_string());
        err
    });

    Ok(result)
}

async fn run_exporter_query(
    task: &BeamTask,
    body: &String,
    task_type: exporter::TaskType,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let exporter_result = exporter::post_exporter_query(body, task_type).await?;

    let result = beam_result(task.to_owned(), exporter_result).unwrap_or_else(|e| {
        err.body = beam_lib::RawString(e.to_string());
        err
    });

    Ok(result)
}

fn replace_cql_library(mut query: CqlQuery) -> Result<CqlQuery, FocusError> {
    let old_data_value = &query.lib["content"][0]["data"];

    let old_data_string = old_data_value
        .as_str()
        .ok_or(FocusError::ParsingError(format!(
            "{} is not a valid library: Field .content[0].data not found.",
            query.lib
        )))?;

    let decoded_cql = util::base64_decode(old_data_string)?;

    let decoded_string = str::from_utf8(&decoded_cql)
        .map_err(|_| FocusError::ParsingError("CQL query was invalid".into()))?;

    match is_cql_tampered_with(decoded_string) {
        false => debug!("CQL not tampered with"),
        true => {
            debug!("CQL tampered with");
            return Err(FocusError::CQLTemperedWithError(
                "'define' keyword found in CQL".to_string(),
            ));
        }
    };

    let replaced_cql_str = util::replace_cql(decoded_string);
    let replaced_cql_str_base64 = BASE64.encode(replaced_cql_str);
    let new_data_value = serde_json::to_value(replaced_cql_str_base64)
        .expect("unable to turn base64 string into json value - this should not happen");

    let a = &mut query.lib["content"][0]["data"];
    *a = new_data_value;

    Ok(query)
}

fn beam_result(task: BeamTask, measure_report: String) -> Result<BeamResult, FocusError> {
    let data = BASE64.encode(measure_report.as_bytes());
    Ok(beam::beam_result::succeeded(
        CONFIG.beam_app_id_long.clone(),
        vec![task.from],
        task.id,
        data,
    ))
}


#[cfg(test)]
mod test {
    use super::*;

    const METADATA_STRING: &str = r#"{"project": "exliquid"}"#;
    const METADATA_STRING_EXPORTER: &str = r#"{"project": "exporter", "task_type": "EXECUTE"}"#;

    #[test]
    fn test_metadata_deserialization_default() {
        let metadata: Metadata = serde_json::from_str(METADATA_STRING).unwrap_or(Metadata {
            project: "default_obfuscation".to_string(),
            task_type: None
        });

        assert_eq!(metadata.task_type,  None);
    }

    #[test]
    fn test_metadata_deserialization_exporter() {
        let metadata: Metadata = serde_json::from_str(METADATA_STRING_EXPORTER).unwrap();

        assert_eq!(metadata.task_type,  Some(exporter::TaskType::Execute));
    }
}