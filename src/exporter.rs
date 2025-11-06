use base64::{prelude::BASE64_STANDARD as BASE64, Engine as _};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::str;
use tracing::{debug, warn};

use crate::blaze::{parse_blaze_query_payload_ast, CqlQuery, Language};
use crate::config::CONFIG;
use crate::cql;
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

pub async fn post_exporter_query(
    body: &mut String,
    task_type: TaskType,
) -> Result<String, FocusError> {
    let Some(exporter_url) = &CONFIG.exporter_url else {
        return Err(FocusError::MissingExporterEndpoint);
    };

    let mut headers = HeaderMap::new();

    if let Some(api_key) = CONFIG.exporter_api_key.clone() {
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(api_key.as_str()).map_err(FocusError::InvalidHeaderValue)?,
        );
    }

    if task_type == TaskType::Status {
        let value: Value = serde_json::from_slice(&(util::base64_decode(body))?).map_err(|e| {
            FocusError::DeserializationError(format!(r#"Task body is not a valid JSON: {}"#, e))
        })?;
        let id = value["query-execution-id"].as_str();
        let Some(id) = id else {
            return Err(FocusError::ParsingError(format!(
                r#"Body does not contain the id of the query to check the status of: {}"#,
                value
            )));
        };

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

    // as Exporter has no fixed API, we have to drill into the body like this

    let query_format_string: String;
    let ast = "AST".to_string();
    let ast_data = "AST_DATA".to_string();

    if let Ok(query_format) = util::get_json_field(body, "query_format") {
        query_format_string = query_format.to_string();
    } else {
        return Err(FocusError::DeserializationError(
            "No query_format in the body".to_string(),
        ));
    };

    if query_format_string == ast || query_format_string == ast_data {
        debug!("{}", &query_format_string);

        if let Ok(query) = util::get_json_field(body, "query") {
            //this gives us base64 encoded query which contains lang and payload
            let data = util::base64_decode(query.to_string().as_str())?;
            let query: CqlQuery = match serde_json::from_slice::<Language>(&data)? {
                Language::Cql(_cql_query) => {
                    return Err(FocusError::CqlLangNotEnabled); // query_format is AST, can't have CQL in the query then
                }
                Language::Ast(ast_query) => serde_json::from_str(&cql::generate_body(
                    parse_blaze_query_payload_ast(&ast_query.payload)?,
                    crate::projects::Project::Exporter,
                )?)?,
            };

            let mut franken_body = json!(body);
            franken_body["query"] = json!(
                BASE64.encode(serde_json::to_string(&query).expect("Failed to serialize JSON"))
            );
            franken_body["query_format"] = json!(query_format_string.replace("AST", "CQL"));

            *body = serde_json::to_string(&query).expect("Failed to serialize JSON");
        } else {
            return Err(FocusError::DeserializationError(
                "No query in the body".to_string(),
            ));
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
