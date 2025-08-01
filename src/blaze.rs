use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::ast;
use crate::config::CONFIG;
use crate::errors::FocusError;
use crate::util;
use crate::util::get_json_field;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CqlQuery {
    pub lib: Value,
    pub measure: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AstQuery {
    pub payload: String,
}

pub async fn check_availability() -> bool {
    debug!("Checking Blaze availability...");

    let resp = match CONFIG
        .client
        .get(format!("{}metadata", CONFIG.endpoint_url))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            warn!("Error making Blaze request: {:?}", e);
            return false;
        }
    };

    if resp.status().is_success() {
        return true;
    } else {
        warn!(
            "Request to Blaze returned response with non-200 status: {:?}",
            resp
        );
    }
    false
}

pub async fn post_library(library: String) -> Result<(), FocusError> {
    debug!("Creating a Library...");

    let resp = CONFIG
        .client
        .post(format!("{}Library", CONFIG.endpoint_url))
        .header("Content-Type", "application/json")
        .body(library)
        .send()
        .await
        .map_err(FocusError::UnableToPostLibrary)?;

    if resp.status() == StatusCode::CREATED {
        debug!("Successfully created a Library");
    } else {
        let error_message = format!("Error while creating a Library: {}", resp.status());
        warn!("{}", error_message);
    }

    Ok(())
}

pub async fn post_measure(measure: String) -> Result<(), FocusError> {
    debug!("Creating a Measure...");
    let resp = CONFIG
        .client
        .post(format!("{}Measure", CONFIG.endpoint_url))
        .header("Content-Type", "application/json")
        .body(measure)
        .send()
        .await
        .map_err(FocusError::UnableToPostMeasure)?;

    if resp.status() == StatusCode::CREATED {
        debug!("Successfully created a Measure");
    } else {
        let error_message = format!("Error while creating a Measure: {}", resp.status());
        warn!("{}", error_message);
    }

    Ok(())
}

pub async fn evaluate_measure(url: String) -> Result<String, FocusError> {
    debug!("Evaluating the Measure with canonical URL: {}", url);
    let resp = CONFIG
        .client
        .get(format!(
            "{}Measure/$evaluate-measure?measure={}&periodStart=2000&periodEnd=2030",
            CONFIG.endpoint_url, url
        ))
        .send()
        .await
        .map_err(FocusError::MeasureEvaluationErrorReqwest)?;

    if resp.status() == StatusCode::OK {
        info!(
            "Successfully evaluated the Measure with canonical URL: {}",
            url
        );
        resp.text()
            .await
            .map_err(FocusError::MeasureEvaluationErrorReqwest)
    } else {
        warn!(
            "Error while evaluating the Measure with canonical URL `{}`: {:?}",
            url, resp
        );
        Err(FocusError::MeasureEvaluationErrorBlaze(format!(
            "Error while evaluating the Measure with canonical URL `{}`: {:?}",
            url, resp
        )))
    }
}

pub async fn run_cql_query(library: &Value, measure: &Value) -> Result<String, FocusError> {
    let url: String = if let Ok(value) = get_json_field(&measure.to_string(), "url") {
        value.to_string().replace('"', "")
    } else {
        return Err(FocusError::CQLQueryError);
    };
    debug!("Evaluating the Measure with canonical URL: {}", url);

    post_library(library.to_string()).await?; //TODO make it with into or could change the function signature to take the library
    post_measure(measure.to_string()).await?; //ditto   &str
    evaluate_measure(url).await
}

pub fn parse_blaze_query_payload_ast(ast_query: &str) -> Result<ast::Ast, FocusError> {
    let decoded = util::base64_decode(ast_query)?;
    Ok(serde_json::from_slice(&decoded)?)
}
