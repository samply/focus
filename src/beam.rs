use std::time::Duration;

use beam_lib::{TaskResult, BeamClient, BlockingOptions, MsgId, TaskRequest};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, warn};

use crate::{config::CONFIG, errors::FocusError};

pub mod beam_result {
    use beam_lib::{AppId, TaskResult, WorkStatus, MsgId};
    use serde::Serialize;
    use serde_json::Value;

    pub fn claimed(from: AppId, to: Vec<AppId>, task_id: MsgId) -> TaskResult<()> {
        TaskResult {
            from,
            to,
            task: task_id,
            status: WorkStatus::Claimed,
            metadata: Value::Null,
            body: (),
        }
    }
    pub fn succeeded<T: Serialize>(from: AppId, to: Vec<AppId>, task_id: MsgId, body: T) -> TaskResult<T> {
        TaskResult {
            from,
            to,
            task: task_id,
            status: WorkStatus::Succeeded,
            metadata: Value::Null,
            body,
        }
    }

    pub fn perm_failed<T: Serialize>(from: AppId, to: Vec<AppId>, task_id: MsgId, body: T) -> TaskResult<T> {
        TaskResult {
            from,
            to,
            task: task_id,
            status: WorkStatus::PermFailed,
            metadata: Value::Null,
            body,
        }
    }
}

pub async fn check_availability() -> bool {
    debug!("Checking Beam availability...");

    let resp = match CONFIG.client
        .get(format!("{}v1/health", CONFIG.beam_proxy_url))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            warn!("Error making Beam request: {:?}", e);
            return false;
        }
    };

    if resp.status().is_success() {
        debug!("Beam is available now.");
        return true;
    }
    false
}

const BEAM_CLIENT: Lazy<BeamClient> = Lazy::new(|| BeamClient::new(
    &CONFIG.beam_app_id_long,
    &CONFIG.api_key,
    CONFIG.beam_proxy_url.to_string().parse().expect("Uri always converts to url")
));

pub async fn retrieve_tasks<T: DeserializeOwned>() -> Result<Vec<TaskRequest<T>>, FocusError> {
    debug!("Retrieving tasks...");
    let block = BlockingOptions {
        wait_time: Some(Duration::from_secs(10)),
        wait_count: Some(1)
    };
    BEAM_CLIENT.poll_pending_tasks(&block)
        .await
        .map_err(FocusError::UnableToRetrieveTasksHttp)
}

pub async fn answer_task<T: Serialize>(task_id: MsgId, result: &TaskResult<T>) -> Result<(), FocusError> {
    debug!("Answer task with id: {task_id}");
    BEAM_CLIENT.put_result(result, &task_id)
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))
}

pub async fn fail_task<T>(task: &TaskRequest<T>, body: impl Into<String>) -> Result<(), FocusError> {
    let body = body.into();
    warn!("Reporting failed task with id {}: {}", task.id, body);
    let result = beam_result::perm_failed(CONFIG.beam_app_id_long.clone(), vec![task.from.clone()], task.id, body);
    BEAM_CLIENT.put_result(&result, &task.id)
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))
}

pub async fn claim_task<T>(task: &TaskRequest<T>) -> Result<(), FocusError> {
    let result = beam_result::claimed(CONFIG.beam_app_id_long.clone(), vec![task.from.clone()], task.id);
    BEAM_CLIENT.put_result(&result, &task.id)
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))
}
