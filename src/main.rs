mod banner;
mod beam;
mod blaze;
mod config;
mod logger;
mod util;

use std::process::exit;

use base64::{engine::general_purpose, Engine as _};
use beam::{BeamResult, BeamTask};
use blaze::Inquery;
use serde_json::from_slice;

use tracing::{debug, error};

use crate::{config::CONFIG, errors::SpotError};

mod errors;

#[tokio::main]
async fn main() -> Result<(), SpotError> {
    if let Err(e) = logger::init_logger() {
        error!("Cannot initalize logger: {}", e);
        exit(1);
    };
    banner::print_banner();

    let _ = CONFIG.api_key; // Initialize config

    beam::check_availability().await;
    blaze::check_availability().await;

    loop {
        process_tasks().await?;
    }
}

async fn process_tasks() -> Result<(), SpotError> {
    debug!("Start processing tasks...");

    let tasks = beam::retrieve_tasks().await?;
    for task in tasks {
        debug!("Processing task with ID: {}", task.id);

        let inquery = parse_inquery(&task)?;
        let run_result = run_inquery(&task, &inquery).await?;
        beam::answer_task(task, run_result).await?;
    }
    Ok(())
}

fn parse_inquery(task: &BeamTask) -> Result<blaze::Inquery, SpotError> {
    let decoded = general_purpose::STANDARD
        .decode(task.body.clone())
        .map_err(|e| SpotError::DecodeError(e))?;
    let inquery: blaze::Inquery =
        from_slice(&decoded).map_err(|e| SpotError::ParsingError(e.to_string()))?;
    Ok(inquery)
}

async fn run_inquery(task: &BeamTask, inquery: &Inquery) -> Result<BeamResult, SpotError> {
    if inquery.lang == "cql" {
        // TODO: Change inquery.lang to an enum
        return Ok(run_cql_query(task, inquery).await);
    } else {
        return Ok(beam::BeamResult::perm_failed(
            CONFIG.beam_app_id.clone(),
            vec![task.from.closne()],
            task.id,
            format!("Can't run inqueries with language {}", inquery.lang),
        ));
    }
}

async fn run_cql_query(task: &BeamTask, inquery: &Inquery) -> BeamResult {
    let mut err = beam::BeamResult::perm_failed(
        CONFIG.beam_app_id.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );
    let cql_result = match blaze::run_cql_query(&inquery.lib, &inquery.measure).await {
        Ok(s) => s,
        Err(e) => {
            err.body = e.to_string();
            return err;
        }
    };
    let result = beam_result(task.to_owned(), cql_result).unwrap_or_else(|e| {
        err.body = e.to_string();
        return err;
    });
    result
}

fn beam_result(
    task: beam::BeamTask,
    measure_report: String,
) -> Result<beam::BeamResult, SpotError> {
    let data = general_purpose::STANDARD.encode(measure_report.as_bytes());
    return Ok(beam::BeamResult::succeeded(
        CONFIG.beam_app_id.clone(),
        vec![task.from],
        task.id,
        data,
    ));
}
