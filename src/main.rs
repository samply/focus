mod banner;
mod beam;
mod blaze;
mod config;
mod logger;
mod util;

use std::collections::HashMap;
use std::process::ExitCode;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{process::exit, time::Duration};

use base64::{engine::general_purpose, Engine as _};
use beam::{BeamResult, BeamTask};
use blaze::Query;
use serde_json::from_slice;

use tracing::{debug, error, info, warn};

use crate::util::{is_cql_tampered_with, obfuscate_counts_mr};
use crate::{config::CONFIG, errors::FocusError};

use laplace_rs::ObfCache;

mod errors;
mod graceful_shutdown;

// result cache
type SearchQuery = String;
type Report = String;
type Created = std::time::SystemTime; //epoch

#[derive(Debug, Clone)]
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
    warn!("asdfasdfafd");
    warn!("asdfasdfafd");
    warn!("asdfasdfafd");
    warn!("asdfasdfafd");
    warn!("asdfasdfafd");
    let mut obf_cache: ObfCache = ObfCache {
        cache: HashMap::new(),
    };

    let mut report_cache: ReportCache = ReportCache {
        cache: HashMap::new(),
    };

    match CONFIG.queries_to_cache_file_path.clone() {
        Some(filename) => {
            let lines = util::read_lines(filename.clone().to_string());

            match lines {
                Ok(ok_lines) => {
                    for line in ok_lines {
                        let Ok(ok_line) = line else{
                            warn!("A line in the file {} is not readable", filename);
                            continue;
                        };
                        report_cache.cache.insert(ok_line, ("".into(), UNIX_EPOCH));
                    }
                }
                Err(_) => {
                    error!("The file {} cannot be opened", filename);
                    exit(2);
                }
            }
        }
        None => {}
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
        if !(beam::check_availability().await && blaze::check_availability().await) {
            failures += 1;
        }
        if let Err(e) = process_tasks(&mut obf_cache, &mut report_cache).await {
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
    obf_cache: &mut ObfCache,
    report_cache: &mut ReportCache,
) -> Result<BeamResult, FocusError> {
    debug!("Processing task {}", task.id);

    let query = parse_query(task)?;

    let run_result = run_query(task, &query, obf_cache, report_cache).await?;

    info!("Reported successful execution of task {} to Beam.", task.id);

    Ok(run_result)
}

async fn process_tasks(
    obf_cache: &mut ObfCache,
    report_cache: &mut ReportCache,
) -> Result<(), FocusError> {
    debug!("Start processing tasks...");

    let tasks = beam::retrieve_tasks().await?;
    for task in tasks {
        let task_cloned = task.clone();
        let claiming = tokio::task::spawn(async move {
            beam::claim_task(&task_cloned).await
        });
        let res = process_task(&task, obf_cache, report_cache).await;
        let error_msg = match res {
            Err(FocusError::DecodeError(_)) | Err(FocusError::ParsingError(_)) => {
                Some("Cannot parse query".to_string())
            }
            Err(FocusError::LaplaceError(_)) => Some("Cannot obfuscate result".to_string()),
            Err(ref e) => Some(format!("Cannot execute query: {}", e)),
            Ok(_) => None,
        };

        let res = res.ok();
        let res = res.as_ref();

        // Make sure that claiming the task is done before we update it again.
        match claiming.await.unwrap() {
            Ok(_) => {},
            Err(FocusError::ConfigurationError(s)) => {
                error!("FATAL: Unable to report back to Beam due to a configuration issue: {s}");
                return Err(FocusError::ConfigurationError(s));
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
                beam::answer_task(&task, res.unwrap()).await
            };
            match comm_result {
                Ok(_) => break,
                Err(FocusError::ConfigurationError(s)) => {
                    error!(
                        "FATAL: Unable to report back to Beam due to a configuration issue: {s}"
                    );
                    return Err(FocusError::ConfigurationError(s));
                }
                Err(FocusError::UnableToAnswerTask(e)) => {
                    warn!("Unable to report task result to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
                }
                Err(e) => {
                    warn!("Unknown error reporting task result back to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    Ok(())
}

fn parse_query(task: &BeamTask) -> Result<blaze::Query, FocusError> {
    let decoded = general_purpose::STANDARD
        .decode(task.body.to_owned())
        .map_err(|e| FocusError::DecodeError(e))?;

    let query: blaze::Query =
        from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;

    Ok(query)
}

async fn run_query(
    task: &BeamTask,
    query: &Query,
    obf_cache: &mut ObfCache,
    report_cache: &mut ReportCache,
) -> Result<BeamResult, FocusError> {
    debug!("Run");

    if query.lang == "cql" {
        // TODO: Change query.lang to an enum
        return Ok(run_cql_query(task, query, obf_cache, report_cache).await)?;
    } else {
        return Ok(beam::BeamResult::perm_failed(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            format!("Can't run inqueries with language {}", query.lang),
        ));
    }
}

async fn run_cql_query(
    task: &BeamTask,
    query: &Query,
    obf_cache: &mut ObfCache,
    report_cache: &mut ReportCache,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::BeamResult::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let encoded_query =
        query.lib["content"][0]["data"]
            .as_str()
            .ok_or(FocusError::ParsingError(format!(
                "Not a valid library: Field .content[0].data not found. Library: {}",
                query.lib.to_string()
            )))?;

    let mut key_exists = false;

    let cached_report = report_cache.cache.get(encoded_query);
    let report_from_cache = match cached_report {
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
                config::Obfuscate::Yes => obfuscate_counts_mr(
                    &cql_result,
                    obf_cache,
                    CONFIG.obfuscate_zero,
                    CONFIG.obfuscate_below_10_mode,
                    CONFIG.delta_patient,
                    CONFIG.delta_specimen,
                    CONFIG.delta_diagnosis,
                    CONFIG.epsilon,
                    CONFIG.rounding_step,
                )?,
                config::Obfuscate::No => cql_result,
            };

            if key_exists {
                report_cache.cache.insert(
                    encoded_query.to_string(),
                    (cql_result_new.clone(), std::time::SystemTime::now()),
                );
            }
            cql_result_new
        }
    };

    let result = beam_result(task.to_owned(), cql_result_new).unwrap_or_else(|e| {
        err.body = e.to_string();
        return err;
    });

    Ok(result)
}

fn replace_cql_library(mut query: Query) -> Result<Query, FocusError> {
    let old_data_value = &query.lib["content"][0]["data"];

    let old_data_string = old_data_value
        .as_str()
        .ok_or(FocusError::ParsingError(format!(
            "{} is not a valid library: Field .content[0].data not found.",
            query.lib.to_string()
        )))?;

    let decoded_cql = general_purpose::STANDARD
        .decode(old_data_string)
        .map_err(|e| FocusError::DecodeError(e))?;

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

fn beam_result(
    task: beam::BeamTask,
    measure_report: String,
) -> Result<beam::BeamResult, FocusError> {
    let data = general_purpose::STANDARD.encode(measure_report.as_bytes());
    return Ok(beam::BeamResult::succeeded(
        CONFIG.beam_app_id_long.clone(),
        vec![task.from],
        task.id,
        data,
    ));
}
