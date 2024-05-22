use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::str;
use tracing::{debug, warn};

use crate::config::CONFIG;
use crate::errors::FocusError;
use crate::util;

#[derive(Clone, PartialEq, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskType {
    Execute,
    Create,
    Status,
}

struct Params {
    method: &'static str,
    doing: &'static str,
    done: &'static str,
}

const CREATE: Params = Params {
    method: "create-query",
    doing: "creating",
    done: "created",
};

const EXECUTE: Params = Params {
    method: "request",
    doing: "executing",
    done: "executed",
};

pub async fn post_exporter_query(body: &String, task_type: TaskType) -> Result<String, FocusError> {
    let Some(exporter_url) = &CONFIG.exporter_url else {
        return Err(FocusError::MissingExporterEndpoint());
    };

    let mut headers = HeaderMap::new();

    if let Some(auth_header_value) = CONFIG.auth_header.clone() {
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(auth_header_value.as_str())
                .map_err(FocusError::InvalidHeaderValue)?,
        );
    }

    if task_type == TaskType::Status {
        let value: Value = serde_json::from_str(
            String::from_utf8(util::base64_decode(&body)?)
                .map_err(|e| {
                    FocusError::DeserializationError(format!(
                        r#"Task body is not a valid string {}"#,
                        e
                    ))
                })?
                .as_str(),
        )
        .map_err(|e| {
            FocusError::DeserializationError(format!(r#"Task body is not a valid JSON: {}"#, e))
        })?;
        let id = value["query-execution-id"].as_str();
        if id.is_none() {
            return Err(FocusError::ParsingError(format!(
                r#"Body does not contain the id of the query to check the status of: {}"#,
                value
            )));
        }
        let id: &str = id.unwrap(); //we already made sure that it is not None

        let resp = CONFIG
            .client
            .get(format!("{}status?query-execution-id={}", exporter_url, id))
            .headers(headers)
            .send()
            .await
            .map_err(FocusError::UnableToGetExporterQueryStatus)?;

        debug!("asked for status for query id= {} ", id);

        match resp.status() {
            StatusCode::OK => {
                let text = resp.text().await;
                match text {
                    Ok(ok_text) => {
                        return Ok(ok_text);
                    }
                    Err(e) => {
                        warn!(
                        "The code was 200 OK, but can't get the body of the Exporter's response for status of the query id={}, {}", id, e);
                        return Err(FocusError::ExporterQueryErrorReqwest(format!(
                        "Error while checking the status of the query id={}, the code was 200 OK, but can't get the body of the Exporter's response: {}",
                        id, e
                    )));
                    }
                }
            }
            code => {
                warn!(
                    "Got unexpected code {code} while checking the status of the query id={}, {:?}",
                    id, resp
                );
                return Err(FocusError::ExporterQueryErrorReqwest(format!(
                    "Error while checking the status of the query id={}, {:?}",
                    id, resp
                )));
            }
        };
    }

    let exporter_params = if task_type == TaskType::Execute {
        EXECUTE
    } else {
        CREATE
    };
    debug!("{} exporter query...", exporter_params.doing);

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let resp = CONFIG
        .client
        .post(format!("{}{}", exporter_url, exporter_params.method))
        .headers(headers)
        .body(body.clone())
        .send()
        .await
        .map_err(FocusError::UnableToPostExporterQuery)?;

    debug!("{} query...", exporter_params.done);

    let text = match resp.status() {
        StatusCode::OK => {
            let text = resp.text().await;
            match text {
                Ok(ok_text) => ok_text,
                Err(e) => {
                    warn!(
                        "The code was 200 OK, but can't get the body of the Exporter's response, while {} query; reply was `{}`, error: {}",
                        exporter_params.doing, body, e
                    );
                    return Err(FocusError::ExporterQueryErrorReqwest(format!(
                        "Error while {} query, the code was 200 OK, but can't get the body of the Exporter's response: {:?}",
                        exporter_params.doing, body
                    )));
                }
            }
        }
        code => {
            warn!(
                "Got unexpected code {code} while {} query; reply was `{}`, debug info: {:?}",
                exporter_params.doing, body, resp
            );
            return Err(FocusError::ExporterQueryErrorReqwest(format!(
                "Error while {} query: {:?}",
                exporter_params.doing, resp
            )));
        }
    };

    Ok(text)
}
