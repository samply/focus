use std::{sync::Arc, collections::HashMap, time::Duration};

use base64::{engine::general_purpose, Engine as _};
use laplace_rs::ObfCache;
use tokio::sync::{mpsc, Semaphore, Mutex};
use tracing::{error, warn, debug, Instrument, info_span};

use crate::{ReportCache, errors::FocusError, beam, BeamTask, BeamResult, run_exporter_query, config::{EndpointType, CONFIG}, run_cql_query, intermediate_rep, ast, run_intermediate_rep_query, Metadata, blaze::parse_blaze_query, util};

const NUM_WORKERS: usize = 3;
const WORKER_BUFFER: usize = 32;

pub type TaskQueue = mpsc::Sender<BeamTask>;

pub fn spawn_task_workers(report_cache: ReportCache) -> TaskQueue {
    let (tx, mut rx) = mpsc::channel::<BeamTask>(WORKER_BUFFER);

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
                let span = info_span!("task handling", %task.id);
                handle_beam_task(task, local_obf_cache, local_report_cache).instrument(span).await;
                drop(permit)
            });
        }
    });

    tx
}

async fn handle_beam_task(task: BeamTask, local_obf_cache: Arc<Mutex<ObfCache>>, local_report_cache: Arc<Mutex<ReportCache>>) {
    let task_claiming = beam::claim_task(&task);
    let mut task_processing = std::pin::pin!(process_task(&task, local_obf_cache, local_report_cache));
    let task_result = tokio::select! {
        // If task task processing happens before claiming is done drop the task claiming future  
        task_processed = &mut task_processing => {
            task_processed
        },
        task_claimed = task_claiming => {
            if let Err(e) = task_claimed {
                warn!("Failed to claim task: {e}");
            } else {
                debug!("Successfully claimed task");
            };
            task_processing.await
        }
    };
    let result = match task_result {
        Ok(res) => res,
        Err(e) => {
            warn!("Failed to execute query: {e}");
            if let Err(e) = beam::fail_task(&task, e.user_facing_error()).await {
                warn!("Failed to report failure to beam: {e}");
            }
            return;
        }
    };

    const MAX_TRIES: u32 = 150;
    for attempt in 0..MAX_TRIES {
        match beam::answer_task(&result).await {
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

    if metadata.project == "focus-healthcheck" {
        return Ok(beam::beam_result::succeeded(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            "healthy".into()
        ));
    }

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
