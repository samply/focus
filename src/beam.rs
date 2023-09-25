use std::{fmt::Display};

use http::{HeaderValue, StatusCode};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::{de, Deserializer, Deserialize, Serialize, Serializer};
use tracing::{debug, warn, info};
use uuid::Uuid;

use crate::{config::CONFIG, errors::FocusError};

type BrokerId = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProxyId {
    proxy: String,
    broker: BrokerId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppId {
    app: String,
    rest: ProxyId,
}

impl ProxyId {
    pub fn get_proxy_id(&self) -> String {
        format!("{}.{}", &self.proxy, &self.broker)
    }
    pub fn get_broker_id(&self) -> String {
        self.broker.clone()
    }
    pub fn new(full: String) -> Result<Self, FocusError> {
        let mut components: Vec<String> = full.split(".").map(|x| x.to_string()).collect();
        let rest = components.split_off(1).join(".");
        Ok(ProxyId { 
            proxy: components
                .first()
                .cloned()
                .ok_or_else(|| FocusError::InvalidBeamId(format!("Invalid ProxyId: {}", full)))?,
            broker: rest,
        })
    }
}

impl AppId {
    pub fn get_app_id(&self) -> String {
        format!("{}.{}", &self.app, &self.rest.get_proxy_id())
    }
    pub fn get_proxy_id(&self) -> String {
        self.rest.get_proxy_id()
    }
    pub fn new(full: String) -> Result<Self, FocusError> {
        let mut components: Vec<String> = full.split(".").map(|x| x.to_string()).collect();
        let rest = components.split_off(1).join(".");
        Ok(AppId {
            app: components
                .first()
                .cloned()
                .ok_or_else(|| FocusError::InvalidBeamId(format!("Invalid ProxyId: {}", full)))?,
            rest: ProxyId::new(rest)?,
        })
    }
}
impl Display for ProxyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.proxy, self.broker)
    }
}
impl Display for AppId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.app, self.rest)
    }
}

impl<'de> serde::Deserialize<'de> for AppId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        AppId::new(s).map_err(de::Error::custom)
    }
}

impl Serialize for AppId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
              {
                  let mut state = String::serialize(&self.to_string(), serializer)?;
                  Ok(state)
              }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BeamTask {
    pub id: Uuid,
    pub from: AppId,
    pub to: Vec<AppId>,
    pub metadata: String,
    pub body: String,
    pub ttl: String,
    pub failure_strategy: FailureStrategy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FailureStrategy {
    Retry(Retry),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Retry {
    pub backoff_millisecs: usize,
    pub max_tries: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BeamResult {
    pub from: AppId,
    pub to: Vec<AppId>,
    pub task: Uuid,
    pub status: Status,
    pub metadata: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Claimed,
    Succeeded,
    TempFailed,
    PermFailed,
}

impl BeamResult {
    pub fn claimed(from: AppId, to: Vec<AppId>, task: Uuid) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::Claimed,
            metadata: None,
            body: None,
        }
    }
    pub fn succeeded(from: AppId, to: Vec<AppId>, task: Uuid, body: Option<String>) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::Succeeded,
            metadata: None,
            body,
        }
    }

    pub fn perm_failed(from: AppId, to: Vec<AppId>, task: Uuid, body: Option<String>) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::PermFailed,
            metadata: None,
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

pub async fn retrieve_tasks() -> Result<Vec<BeamTask>, FocusError> {
    debug!("Retrieving tasks...");

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("ApiKey {} {}", CONFIG.beam_app_id_long, CONFIG.api_key))
            .map_err(|e| {
                FocusError::ConfigurationError(format!(
                    "Cannot assemble authorization header: {}",
                    e
                ))
            })?,
    );

    let url = format!(
        "{}v1/tasks?filter=todo&wait_count=1&wait_time=10s",
        CONFIG.beam_proxy_url
    );
    let resp = CONFIG.client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| FocusError::UnableToRetrieveTasksHttp(e))?;

    let tasks = match resp.status() {
        StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
            resp
                .json::<Vec<BeamTask>>()
                .await
                .map_err(|e| FocusError::UnableToParseTasks(e))?
        }
        code => {
            return Err(FocusError::UnableToRetrieveTasksOther(format!("Got status code {code}")));
        }
    };
    Ok(tasks)
}

pub async fn answer_task(task: &BeamTask, result: &BeamResult) -> Result<(), FocusError> {
    let task_id = task.id.to_string();
    debug!("Answer task with id: {task_id}");
    let result_task = result.task;
    let url = format!(
        "{}v1/tasks/{}/results/{}",
        CONFIG.beam_proxy_url, &result_task, CONFIG.beam_app_id_long
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("ApiKey {} {}", CONFIG.beam_app_id_long, CONFIG.api_key))
            .map_err(|e| {
                FocusError::ConfigurationError(format!(
                    "Cannot assemble authorization header: {}",
                    e
                ))
            })?,
    );

    let resp = CONFIG.client
        .put(&url)
        .headers(headers)
        .json(&result)
        .send()
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))?;

    let status_code = resp.status();
    let status_text = status_code.as_str();
    debug!("{status_text}");

    match status_code {
        StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::BAD_REQUEST => {
            let msg = resp
                .text()
                .await
                .map_err(|e| FocusError::UnableToAnswerTask(e))?;
            warn!("Error while answering the task with id: {msg}");
            Ok(()) // return error
        }
        _ => {
            warn!("Unexpected status code: {}", resp.status());
            Ok(()) //return error
        }
    }
}

pub async fn fail_task(task: &BeamTask, body: impl Into<String>) -> Result<(), FocusError> {
    let body = body.into();
    warn!("Reporting failed task with id {}: {}", task.id, body);
    let result = BeamResult::perm_failed(CONFIG.beam_app_id_long.clone(), vec![task.from.clone()], task.id, Some(body));
    let url = format!(
        "{}v1/tasks/{}/results/{}",
        CONFIG.beam_proxy_url, task.id, CONFIG.beam_app_id_long
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("ApiKey {} {}", CONFIG.beam_app_id_long, CONFIG.api_key))
            .map_err(|e| {
                FocusError::ConfigurationError(format!(
                    "Cannot assemble authorization header: {}",
                    e
                ))
            })?,
    );

    let resp = CONFIG.client
        .put(&url)
        .headers(headers)
        .json(&result)
        .send()
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))?;

    match resp.status() {
        StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::BAD_REQUEST => {
            let msg = resp
                .text()
                .await
                .map_err(|e| FocusError::UnableToAnswerTask(e))?;
            warn!("Error while reporting the failed task with id {}: {msg}", task.id);
            Ok(()) // return error
        }
        _ => {
            warn!("Unexpected status code: {}", resp.status());
            Ok(()) //return error
        }
    }
}

pub async fn claim_task(task: &BeamTask) -> Result<(), FocusError> {
    let result = BeamResult::claimed(CONFIG.beam_app_id_long.clone(), vec![task.from.clone()], task.id);
    let url = format!(
        "{}v1/tasks/{}/results/{}",
        CONFIG.beam_proxy_url, task.id, CONFIG.beam_app_id_long
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("ApiKey {} {}", CONFIG.beam_app_id_long, CONFIG.api_key))
            .map_err(|e| {
                FocusError::ConfigurationError(format!(
                    "Cannot assemble authorization header: {}",
                    e
                ))
            })?,
    );

    let resp = CONFIG.client
        .put(&url)
        .headers(headers)
        .json(&result)
        .send()
        .await
        .map_err(|e| FocusError::UnableToAnswerTask(e))?;

    match resp.status() {
        StatusCode::CREATED | StatusCode::NO_CONTENT => {
            info!("Task {} claimed", task.id);
            Ok(())
        },
        StatusCode::BAD_REQUEST => {
            let msg = resp
                .text()
                .await
                .map_err(|e| FocusError::UnableToAnswerTask(e))?;
            warn!("Error while reporting the claimed task with id {}: {msg}", task.id);
            Ok(()) // return error
        }
        _ => {
            warn!("Unexpected status code: {}", resp.status());
            Ok(()) //return error
        }
    }
}
