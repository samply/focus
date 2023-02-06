use anyhow::Result;
use http::{StatusCode, HeaderValue};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tokio::time::Duration;
use uuid::Uuid;
use std::collections::HashMap;

use crate::errors::SpotError;

const MAX_HEALTH_CHECK_ATTEMPTS: i32 = 5;
const BEAM_BASE_URL: &str = "http://localhost:8081";
pub(crate) const PROXY_ID: &str = "proxy1.broker";
const API_KEY: &str = "App1Secret";
pub const APP_ID: &str = "app1";



#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Inquery {
    pub lang: String,
    pub lib: Value,
    pub measure: Value
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub struct BeamTask {
    pub id: Uuid,
    pub from: String,
    pub to: Vec<String>,
    pub metadata: String,
    pub body: String,
    pub ttl: usize,
    pub failure_strategy: FailureStrategy,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub struct FailureStrategy {
    pub retry: Retry,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Retry {
    pub backoff_millisecs: i32,
    pub max_tries: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct BeamResult {
    pub from: String,
    pub to: Vec<String>,
    pub task: Uuid,
    pub status: Status,
    pub metadata: String,
    pub body: String,
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
    pub fn claimed(from: String, to: Vec<String>, task: Uuid) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::Claimed,
            metadata: "foo".to_owned(),
            body: "".to_owned(),
        }
    }

    pub fn succeeded(
        from: String,
        to: Vec<String>,
        task: Uuid,
        body: String,
        app_id: String,
    ) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::Succeeded,
            metadata: app_id,
            body,
        }
    }

    pub fn perm_failed(
        from: String,
        to: Vec<String>,
        task: Uuid,
        body: String,
        app_id: String,
    ) -> Self {
        Self {
            from,
            to,
            task,
            status: Status::PermFailed,
            metadata: app_id,
            body,
        }
    }
}

pub async fn check_availability() {
    let client = Client::new();
    let mut attempt = 0;

    println!("Check Beam availability...");

    loop {
        let resp = match client
            .get(BEAM_BASE_URL.to_owned() + "/v1/health")
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                println!("Error making request: {:?}", e);
                continue;
            }
        };

        let status_code = resp.status();
        let status_text = status_code.as_str();
        println!("status:  {status_text}");

        if resp.status().is_success() {
            println!("Beam is available now.");
            break;
        } else if attempt == MAX_HEALTH_CHECK_ATTEMPTS {
            println!("Beam still not available after {MAX_HEALTH_CHECK_ATTEMPTS} attempts.");
            break;
        } else {
            println!("Beam still not available, retrying in 3 seconds...");
            sleep(Duration::from_secs(3)).await;
            attempt += 1;
        }
    }
}

pub async fn retrieve_tasks() -> Result<Vec<BeamTask>, SpotError> {
    println!("Retrieve tasks...");

    let client = reqwest::Client::new();
    let mut tasks: Vec<BeamTask> = Vec::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("ApiKey {}.{} {}", APP_ID, PROXY_ID, API_KEY)).map_err(|e| SpotError::ConfigurationError(format!("Cannot assemble authorization header: {}", e)))?);

    let url = BEAM_BASE_URL.to_owned() + "/v1/tasks?filter=todo&wait_count=1&wait_time=10000";
    println!("Header: ApiKey {}.{} {}", APP_ID, PROXY_ID, API_KEY);
    let resp = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| SpotError::UnableToRetrieveTasks(e))?;

    let status_code = resp.status();
    let status_text = status_code.as_str();
    println!("status:  {status_text}");

    match status_code {
        StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
            tasks = resp
                .json::<Vec<BeamTask>>()
                .await
                .map_err(|e| SpotError::UnableToParseTasks(e))?;
        }
        _ => {
            println!("Unable to retrieve tasks: {}", status_code);
            //return error
        }
    }
    println!("{:?}", tasks);
    Ok(tasks)
}

pub async fn answer_task(task: BeamTask, result: BeamResult) -> Result<(), SpotError> {
    let task_id = task.id.to_string();
    println!("Answer task with id: {task_id}");
    let result_task = result.task;
    let url = format!("{}/v1/tasks/{}/results/{}.{}",BEAM_BASE_URL, &result_task, APP_ID, PROXY_ID);

    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("ApiKey {}.{} {}", APP_ID, PROXY_ID, API_KEY)).map_err(|e| SpotError::ConfigurationError(format!("Cannot assemble authorization header: {}", e)))?);

    let resp = client
        .put(&url)
        .headers(headers)
        .json(&result)
        .send()
        .await
        .map_err(|e| SpotError::UnableToAnswerTask(e))?;

    let status_code = resp.status();
    let status_text = status_code.as_str();
    println!("status:  {status_text}");

    match status_code {
        StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
        StatusCode::BAD_REQUEST => {
            let msg = resp
                .text()
                .await
                .map_err(|e| SpotError::UnableToAnswerTask(e))?;
            println!("Error while answering the task with id: {msg}");
            Ok(()) // return error
            

        }
        _ => {
            let msg = format!("Unexpected status code: {}", resp.status());
            println!("{}", msg);
            Ok(()) //return error
        }
    }
}
