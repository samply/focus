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

use tracing::{debug, warn, error};

use crate::{config::CONFIG, errors::FocusError};

mod errors;

#[tokio::main]
async fn main() -> Result<(), FocusError> {
    if let Err(e) = logger::init_logger() {
        error!("Cannot initalize logger: {}", e);
        exit(1);
    };
    banner::print_banner();

    let _ = CONFIG.api_key; // Initialize config

    loop {
        beam::check_availability().await;
        blaze::check_availability().await;
        if let Err(e) = process_tasks().await {
            warn!("Encontered the following error, while processing tasks: {e}");
        };
    }
}

async fn process_tasks() -> Result<(), FocusError> {
    debug!("Start processing tasks...");

    let tasks = beam::retrieve_tasks().await?;
    for task in tasks {
        debug!("Processing task with ID: {}", task.id);

        let Ok(inquery) = parse_inquery(&task) else {beam::fail_task(task, "Cannot pare inquery".into()).await?; continue;};
        let Ok(run_result) = run_inquery(&task, &inquery).await else {beam::fail_task(task, "Cannot run inquery".into()).await?; continue;};
        match beam::answer_task(task.clone(), run_result).await {Ok(()) => (), Err(e) => {beam::fail_task(task, "e".to_string()).await?; continue;}};
    }
    Ok(())
}

fn parse_inquery(task: &BeamTask) -> Result<blaze::Inquery, FocusError> {
    let decoded = general_purpose::STANDARD
        .decode(task.body.clone())
        .map_err(|e| FocusError::DecodeError(e))?;
    let inquery: blaze::Inquery =
        from_slice(&decoded).map_err(|e| FocusError::ParsingError(e.to_string()))?;
    Ok(inquery)
}

async fn run_inquery(task: &BeamTask, inquery: &Inquery) -> Result<BeamResult, FocusError> {
    debug!("Run");
    if inquery.lang == "cql" {
        // TODO: Change inquery.lang to an enum
        return Ok(run_cql_query(task, inquery).await);
    } else {
        return Ok(beam::BeamResult::perm_failed(
            CONFIG.beam_app_id.clone(),
            vec![task.from.clone()],
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
) -> Result<beam::BeamResult, FocusError> {
    let data = general_purpose::STANDARD.encode(measure_report.as_bytes());
    return Ok(beam::BeamResult::succeeded(
        CONFIG.beam_app_id.clone(),
        vec![task.from],
        task.id,
        data,
    ));
}
