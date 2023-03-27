mod banner;
mod beam;
mod blaze;
mod config;
mod logger;
mod util;

use std::process::ExitCode;
use std::str;
use std::{process::exit, time::Duration};
use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use beam::{BeamResult, BeamTask};
use blaze::Query;
use serde_json::from_slice;

use tracing::{debug, error, warn, info};

use crate::util::{is_cql_tampered_with};
use crate::{config::CONFIG, errors::FocusError};

use laplace_rs::{ObfCache, obfuscate_counts};

mod errors;


#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = logger::init_logger() {
        error!("Cannot initalize logger: {}", e);
        exit(1);
    };
    banner::print_banner();

    let _ = CONFIG.api_key; // Initialize config

    let mut obf_cache: ObfCache = ObfCache { cache: HashMap::new() };

    let mut failures = 0;
    while failures < CONFIG.retry_count {
        if failures > 0 {
            warn!("Retrying connection (attempt {}/{})", failures+1, CONFIG.retry_count);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        if !(beam::check_availability().await && blaze::check_availability().await) {
            failures += 1;
        }
        if let Err(e) = process_tasks(&mut obf_cache).await {
            warn!("Encountered the following error, while processing tasks: {e}");
            warn!("Just to make sure, that 502 could mean just timeout");
            //failures += 1;
        } else {
            failures = 0;
            dbg!(&obf_cache.cache);
        }
    }
    error!("Encountered too many errors -- exiting after {} attempts.", CONFIG.retry_count);
    ExitCode::from(42)
}

async fn process_task(task: &BeamTask, obf_cache: &mut ObfCache) -> Result<BeamResult, FocusError> {
    debug!("Processing task {}", task.id);

    let query = parse_query(task)?;

    let run_result = run_query(task, &query, obf_cache).await?;

    info!("Reported successful execution of task {} to Beam.", task.id);

    Ok(run_result)
}

async fn process_tasks(obf_cache: &mut ObfCache) -> Result<(), FocusError> {
    debug!("Start processing tasks...");

    let tasks = beam::retrieve_tasks().await?;
    for task in tasks {
        let res = process_task(&task, obf_cache).await;
        let error_msg = match res {
            Err(FocusError::DecodeError(_)) | Err(FocusError::ParsingError(_)) => {
                Some("Cannot parse query".to_string())
            }
            Err(FocusError::LaplaceError(_)) => {
                Some("Cannot obfuscate result".to_string())
            }
            Err(ref e) => {
                Some(format!("Cannot execute query: {}", e))
            }
            Ok(_) => None,
        };

        let res = res.ok();
        let res = res.as_ref();

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
    //debug!("{:?}", decoded);


    let query: blaze::Query =
        from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;
    
    Ok(query)
}

async fn run_query(task: &BeamTask, query: &Query, obf_cache: &mut ObfCache) -> Result<BeamResult, FocusError> {
    debug!("Run");

    if query.lang == "cql" {
        // TODO: Change query.lang to an enum
        return Ok(run_cql_query(task, query, obf_cache).await)?;
    } else {
        return Ok(beam::BeamResult::perm_failed(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            format!("Can't run inqueries with language {}", query.lang),
        ));
    }
}

async fn run_cql_query(task: &BeamTask, query: &Query, obf_cache: &mut ObfCache) -> Result<BeamResult, FocusError> {
    let mut err = beam::BeamResult::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let query = replace_cql_library(query.clone())?;

    //dbg!(&query, &query.lib);
    let cql_result = match blaze::run_cql_query(&query.lib, &query.measure).await {
        Ok(s) => s,
        Err(e) => {
            err.body = e.to_string();
            return Err(e);
        }
    };

    //dbg!(&cql_result);

    debug!("_________________________________________________________");
    
    let cql_result_new = obfuscate_counts(&cql_result, obf_cache);

    //dbg!(&cql_result_new);

    //debug!("_________________________________________________________");


    let result = beam_result(task.to_owned(), cql_result_new
    .map_err(|e| FocusError::LaplaceError(e))?
    .to_string()).unwrap_or_else(|e| {
        err.body = e.to_string();
        return err;
    });
    Ok(result)
}



fn replace_cql_library(mut query: Query) -> Result<Query, FocusError> {
    let old_data_value = &query.lib["content"][0]["data"];

    let old_data_string = old_data_value
        .as_str()
        .ok_or(FocusError::ParsingError(format!("{} is not a valid library: Field .content[0].data not found.", query.lib.to_string())))?;

    let decoded_cql = general_purpose::STANDARD
        .decode(old_data_string)
        .map_err(|e| FocusError::DecodeError(e))?;

    let decoded_string = str::from_utf8(&decoded_cql)
        .map_err(|_| FocusError::ParsingError("CQL query was invalid".into()))?;

    match is_cql_tampered_with(decoded_string)
    {
        false => debug!("CQL not tampered with"),
        true => {
            debug!("CQL tampered with");
            return Err(FocusError::CQLTemperedWithError("'define' keyword found in CQL".to_string()));
        }
    };

    let replaced_cql_str = util::replace_cql(decoded_string);
    debug!("{}", replaced_cql_str);

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
