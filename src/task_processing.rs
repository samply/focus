use std::{sync::Arc, collections::HashMap, time::Duration};

use base64::{engine::general_purpose, Engine as _};
use laplace_rs::ObfCache;
use tokio::sync::{mpsc, Semaphore, Mutex};
use tracing::{error, warn, debug};

use crate::{ReportCache, errors::FocusError, beam, BeamTask, BeamResult, run_exporter_query, config::{EndpointType, CONFIG}, run_cql_query, intermediate_rep, ast, run_intermediate_rep_query, Metadata, blaze::parse_blaze_query, util};

const NUM_WORKERS: usize = 3;
const WORKER_BUFFER: usize = 32;

pub type TaskQueue = mpsc::Sender<BeamTask>;

pub fn spawn_task_workers(report_cache: ReportCache) -> TaskQueue {
    let (tx, mut rx) = mpsc::channel(WORKER_BUFFER);

    let obf_cache = Arc::new(Mutex::new(ObfCache {
        cache: HashMap::new(),
    }));

    let report_cache: Arc<Mutex<ReportCache>> = Arc::new(Mutex::new(report_cache));
    
    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(NUM_WORKERS));
        while let Some(task) = rx.recv().await {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let local_report_cache = report_cache.clone();
            let local_obf_cache = obf_cache.clone();
            tokio::spawn(async move {
                handle_beam_task(task, local_obf_cache, local_report_cache).await;
                drop(permit)
            });
        }
    });

    tx
}

async fn handle_beam_task(task: BeamTask, local_obf_cache: Arc<Mutex<ObfCache>>, local_report_cache: Arc<Mutex<ReportCache>>) {
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

async fn process_task(
    task: &BeamTask,
    obf_cache: Arc<Mutex<ObfCache>>,
    report_cache: Arc<Mutex<ReportCache>>,
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
