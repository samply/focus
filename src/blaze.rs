use http::StatusCode;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::errors::FocusError;
use crate::util::get_json_field;
use crate::config::CONFIG;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Inquery {
    pub lang: String,
    pub lib: Value,
    pub measure: Value
}

pub async fn check_availability() {

    debug!("Check Blaze availability...");

    let mut attempt = 0;

    loop {
        let resp = match CONFIG.client
            .get(format!("{}metadata", CONFIG.blaze_url))
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                warn!("Error making request: {:?}", e);
                continue;
            }
        };

        if resp.status().is_success() {
            debug!("Blaze is available now.");
            break;
        } else if attempt == CONFIG.retry_count {
            warn!("Blaze still not available after {} attempts.", CONFIG.retry_count);
            break;
        } else {
            warn!("Blaze still not available, retrying in 3 seconds...");
            sleep(Duration::from_secs(3)).await;
            attempt += 1;
        }
    }
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
        .get(format!(
        "{}Measure/$evaluate-measure?measure={}&periodStart=2000&periodEnd=2030",
        CONFIG.blaze_url,
        url
        ))
        .send()
        .await
        .map_err(|e| FocusError::MeasureEvaluationError(e))?;

    if resp.status() == StatusCode::OK {
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
