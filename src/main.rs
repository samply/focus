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
mod util;

use beam_lib::{TaskRequest, TaskResult, MsgId};
use laplace_rs::ObfCache;
use tokio::sync::Mutex;

use crate::util::{is_cql_tampered_with, obfuscate_counts_mr};
use crate::{config::CONFIG, errors::FocusError};
use blaze::CqlQuery;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::process::ExitCode;
use std::str;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{process::exit, time::Duration};

use base64::{engine::general_purpose, Engine as _};

use serde::{Deserialize, Serialize};
use serde_json::from_slice;

use tracing::{debug, error, warn};

// result cache
type SearchQuery = String;
type Report = String;
type Created = std::time::SystemTime; //epoch
type BeamTask = TaskRequest<String>;
type BeamResult = TaskResult<beam_lib::RawString>;
type Shared<T> = Arc<Mutex<T>>;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Metadata {
    project: String,
    #[serde(default)]
    execute: bool,
}

#[derive(Debug, Clone, Default)]
struct ReportCache {
    cache: HashMap<SearchQuery, (Report, Created)>,
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
    let obf_cache = Arc::new(Mutex::new(ObfCache {
        cache: HashMap::new(),
    }));

    let report_cache: Shared<ReportCache> = Default::default();
    let mut seen_tasks = Default::default();

    if let Some(filename) = CONFIG.queries_to_cache_file_path.clone() {
        let lines = util::read_lines(filename.clone().to_string());
        match lines {
            Ok(ok_lines) => {
                for line in ok_lines {
                    let Ok(ok_line) = line else {
                        warn!("A line in the file {} is not readable", filename);
                        continue;
                    };
                    report_cache.lock().await.cache.insert(ok_line, ("".into(), UNIX_EPOCH));
                }
            }
            Err(_) => {
                error!("The file {} cannot be opened", filename);
                exit(2);
            }
        }
    }

    let mut failures = 0;
    while failures < CONFIG.retry_count {
        if failures > 0 {
            warn!(
                "Retrying connection (attempt {}/{})",
                failures + 1,
                CONFIG.retry_count
            );
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        if !(beam::check_availability().await) {
            failures += 1;
        }
        if CONFIG.endpoint_type == config::EndpointType::Blaze {
            if !(blaze::check_availability().await) {
                failures += 1;
            }
        } else if CONFIG.endpoint_type == config::EndpointType::Omop {

            //TODO health check
        }

        if let Err(e) = process_tasks(obf_cache.clone(),  report_cache.clone(), &mut seen_tasks).await {
            warn!("Encountered the following error, while processing tasks: {e}");
            //failures += 1;
        } else {
            failures = 0;
        }
    }
    error!(
        "Encountered too many errors -- exiting after {} attempts.",
        CONFIG.retry_count
    );
    ExitCode::from(22)
}

async fn process_task(
    task: &BeamTask,
    obf_cache: Shared<ObfCache>,
    report_cache: Shared<ReportCache>,
) -> Result<BeamResult, FocusError> {
    debug!("Processing task {}", task.id);

    let metadata: Metadata = serde_json::from_value(task.metadata.clone()).unwrap_or(Metadata {
        project: "default_obfuscation".to_string(),
        execute: true,
    });

    if metadata.project == "exporter" {
        let body = &task.body;
        return Ok(run_exporter_query(task, body, metadata.execute).await)?;
    }

    if CONFIG.endpoint_type == config::EndpointType::Blaze {
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
    } else if CONFIG.endpoint_type == config::EndpointType::Omop {
        let decoded = decode_body(task)?;
        let intermediate_rep_query: intermediate_rep::IntermediateRepQuery =
            from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;
        //TODO check that the language is ast
        let query_decoded = general_purpose::STANDARD
            .decode(intermediate_rep_query.query)
            .map_err(FocusError::DecodeError)?;
        let ast: ast::Ast =
            from_slice(&query_decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;

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

async fn process_tasks(
    obf_cache: Shared<ObfCache>,
    report_cache: Shared<ReportCache>,
    seen: &mut HashSet<MsgId>,
) -> Result<(), FocusError> {
    debug!("Start processing tasks...");

    let tasks = beam::retrieve_tasks().await?;
    for task in tasks {
        if seen.contains(&task.id) {
            continue;
        }
        seen.insert(task.id);
        let local_report_cache = report_cache.clone();
        let local_obf_cache = obf_cache.clone();
        tokio::spawn(handle_beam_task(task, local_obf_cache, local_report_cache));
    }
    Ok(())
}

async fn handle_beam_task(task: TaskRequest<String>, local_obf_cache: Shared<ObfCache>, local_report_cache: Shared<ReportCache>) {
    let task_cloned = task.clone();
    let claiming = tokio::task::spawn(async move { beam::claim_task(&task_cloned).await });
    let res = process_task(&task, local_obf_cache, local_report_cache).await;
    let error_msg = match res {
        Err(FocusError::DecodeError(_)) | Err(FocusError::ParsingError(_)) => {
            Some("Cannot parse query".to_string())
        }
        Err(FocusError::LaplaceError(_)) => Some("Cannot obfuscate result".to_string()),
        Err(ref e) => Some(format!("Cannot execute query: {}", e)),
        Ok(_) => None,
    };

    let res = res.ok();
    // Make sure that claiming the task is done before we update it again.
    match claiming.await.unwrap() {
        Ok(_) => {}
        Err(FocusError::ConfigurationError(s)) => {
            error!("FATAL: Unable to report back to Beam due to a configuration issue: {s}");
        }
        Err(FocusError::UnableToAnswerTask(e)) => {
            warn!("Unable to report claimed task to Beam: {e}");
        }
        Err(e) => {
            warn!("Unknown error reporting claimed task back to Beam: {e}");
        }
    }

    const MAX_TRIES: u32 = 3600;
    for attempt in 0..MAX_TRIES {
        let comm_result = if let Some(ref err_msg) = error_msg {
            beam::fail_task(&task, err_msg).await
        } else {
            beam::answer_task(task.id, res.as_ref().unwrap()).await
        };
        match comm_result {
            Ok(_) => break,
            Err(FocusError::ConfigurationError(s)) => {
                error!(
                    "FATAL: Unable to report back to Beam due to a configuration issue: {s}"
                );
            }
            Err(FocusError::UnableToAnswerTask(e)) => {
                warn!("Unable to report task result to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
            }
            Err(e) => {
                warn!("Unknown error reporting task result back to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
            }
        };
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

fn decode_body(task: &BeamTask) -> Result<Vec<u8>, FocusError> {
    let decoded = general_purpose::STANDARD
        .decode(&task.body)
        .map_err(FocusError::DecodeError)?;

    Ok(decoded)
}

fn parse_blaze_query(task: &BeamTask) -> Result<blaze::CqlQuery, FocusError> {
    let decoded = decode_body(task)?;

    let query: blaze::CqlQuery =
        from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;

    Ok(query)
}

async fn run_cql_query(
    task: &BeamTask,
    query: &CqlQuery,
    obf_cache: Shared<ObfCache>,
    report_cache: Shared<ReportCache>,
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

    let report_from_cache = match report_cache.lock().await.cache.get(encoded_query) {
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

            let cql_result_new: String = match CONFIG.obfuscate {
                config::Obfuscate::Yes => {
                    if !CONFIG.unobfuscated.contains(&project) {
                        obfuscate_counts_mr(
                            &cql_result,
                            obf_cache.lock().await.deref_mut(),
                            CONFIG.obfuscate_zero,
                            CONFIG.obfuscate_below_10_mode,
                            CONFIG.delta_patient,
                            CONFIG.delta_specimen,
                            CONFIG.delta_diagnosis,
                            CONFIG.delta_procedures,
                            CONFIG.delta_medication_statements,
                            CONFIG.epsilon,
                            CONFIG.rounding_step,
                        )?
                    } else {
                        cql_result
                    }
                }
                config::Obfuscate::No => cql_result,
            };

            if key_exists {
                report_cache.lock().await.cache.insert(
                    encoded_query.to_string(),
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
    execute: bool,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let exporter_result = exporter::post_exporter_query(body, execute).await?;

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

    let decoded_cql = general_purpose::STANDARD
        .decode(old_data_string)
        .map_err(FocusError::DecodeError)?;

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
    let replaced_cql_str_base64 = general_purpose::STANDARD.encode(replaced_cql_str);
    let new_data_value = serde_json::to_value(replaced_cql_str_base64)
        .expect("unable to turn base64 string into json value - this should not happen");

    let a = &mut query.lib["content"][0]["data"];
    *a = new_data_value;

    Ok(query)
}

fn beam_result(task: BeamTask, measure_report: String) -> Result<BeamResult, FocusError> {
    let data = general_purpose::STANDARD.encode(measure_report.as_bytes());
    Ok(beam::beam_result::succeeded(
        CONFIG.beam_app_id_long.clone(),
        vec![task.from],
        task.id,
        data,
    ))
}

mod test {
    use super::*;

    const METADATA_STRING: &str = r#"{"project": "exliquid"}"#;

    #[test]
    fn test_metadata_deserialization_default() {
        let metadata: Metadata = serde_json::from_str(METADATA_STRING).unwrap_or(Metadata {
            project: "default_obfuscation".to_string(),
            execute: true,
        });

        assert!(!metadata.execute);
    }
}
