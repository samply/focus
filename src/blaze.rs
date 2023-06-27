use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::errors::FocusError;
use crate::util::get_json_field;
use crate::config::CONFIG;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Query {
    pub lang: String,
    pub lib: Value,
    pub measure: Value
}

pub async fn check_availability() -> bool {

    debug!("Checking Blaze availability...");

    let resp = match CONFIG.client
        .get(format!("{}metadata", CONFIG.blaze_url))
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
    }
    false
}

pub async fn post_library(library: String) -> Result<(), FocusError> {
    debug!("Creating a Library...");

    let resp = CONFIG.client
        .post(format!("{}Library", CONFIG.blaze_url))
        .header("Content-Type", "application/json")
        .body(library)
        .send()
        .await
        .map_err(|e| FocusError::UnableToPostLibrary(e))?;

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
    let resp = CONFIG.client
        .post(format!("{}Measure", CONFIG.blaze_url))
        .header("Content-Type", "application/json")
        .body(measure)
        .send()
        .await
        .map_err(|e| FocusError::UnableToPostMeasure(e))?;

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
    let mut text: String = String::new();
    let resp = CONFIG.client
        .post(format!("{}Measure/$evaluate-measure?measure={}", CONFIG.blaze_url, url))
        .header("Content-Type", "application/fhir+json")
        .body("{\"resourceType\": \"Parameters\", \"parameter\": [{\"name\": \"periodStart\", \"valueDate\": \"2000\"}, {\"name\": \"periodEnd\", \"valueDate\": \"2030\"}, {\"name\": \"reportType\", \"valueCode\": \"subject-list\"}]}")
        .send()
        .await
        .map_err(|e| FocusError::MeasureEvaluationError(e))?;

    if resp.status() == StatusCode::OK || resp.status() == StatusCode::CREATED {
        debug!(
            "Successfully evaluated the Measure with canonical URL: {}",
            url
        );
        text = resp
            .text()
            .await
            .map_err(|e| FocusError::MeasureEvaluationError(e))?;
    } else {
        warn!(
            "Error while evaluating the Measure with canonical URL `{}`: {:?}",
            url, resp
        );
    }

    Ok(text)
}

pub async fn run_cql_query(library: &Value, measure: &Value) -> Result<String, FocusError> {
    let url: String = if let Ok(value) = get_json_field(&measure.to_string(), "url") {
        value.to_string().replace("\"", "")
    } else {
        return Err(FocusError::CQLQueryError());
    };
    debug!("Evaluating the Measure with canonical URL: {}", url);

    post_library(library.to_string()).await?;
    post_measure(measure.to_string()).await?;
    let result_evaluation = evaluate_measure(url).await;
    return result_evaluation;
}
