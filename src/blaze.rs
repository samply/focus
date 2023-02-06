mod util;
use http::StatusCode;
use reqwest::Client;
use serde_json::Value;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::errors::SpotError;

const BLAZE_BASE_URL: &str = "http://localhost:8089/fhir";

pub async fn check_availability() {
    const MAX_HEALTH_CHECK_ATTEMPTS: i32 = 5;

    println!("Check Blaze availability...");

    let client = Client::new();
    let mut attempt = 0;

    loop {
        let resp = match client
            .get(BLAZE_BASE_URL.to_owned() + "/metadata")
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                println!("Error making request: {:?}", e);
                continue;
            }
        };

        //let status_code = resp.status();
        //let status_text = status_code.as_str();
        //println!("status:  {status_text}");

        if resp.status().is_success() {
            println!("Blaze is available now.");
            break;
        } else if attempt == MAX_HEALTH_CHECK_ATTEMPTS {
            println!("Blaze still not available after {MAX_HEALTH_CHECK_ATTEMPTS} attempts.");
            break;
        } else {
            println!("Blaze still not available, retrying in 3 seconds...");
            sleep(Duration::from_secs(3)).await;
            attempt += 1;
        }
    }
}

pub async fn post_library(library: String) -> Result<(), SpotError> {
    println!("Creating a Library...");

    let client = reqwest::Client::new();
    let resp = client
        .post(BLAZE_BASE_URL.to_owned() + "/Library")
        .header("Content-Type", "application/json")
        .body(library)
        .send()
        .await
        .map_err(|e| SpotError::UnableToPostLibrary(e))?;

    //let status_code = resp.status();
    //let status_text = status_code.as_str();
    //println!("status:  {status_text}");

    if resp.status() == StatusCode::CREATED {
        println!("Successfully created a Library");
    } else {
        let error_message = format!("Error while creating a Library: {}", resp.status());
        println!("{}", error_message);
    }

    Ok(())
}

pub async fn post_measure(measure: String) -> Result<(), SpotError> {
    println!("Creating a Measure...");
    let client = reqwest::Client::new();
    let resp = client
        .post(BLAZE_BASE_URL.to_owned() + "/Measure")
        .header("Content-Type", "application/json")
        .body(measure)
        .send()
        .await
        .map_err(|e| SpotError::UnableToPostMeasure(e))?;

    if resp.status() == StatusCode::CREATED {
        println!("Successfully created a Measure");
    } else {
        let error_message = format!("Error while creating a Measure: {}", resp.status());
        println!("{}", error_message);
    }

    Ok(())
}

pub async fn evaluate_measure(url: String) -> Result<String, SpotError> {
    println!("Evaluating the Measure with canonical URL: {}", url);
    let mut text: String = String::new();
    let resp = reqwest::get(format!(
        "{}/Measure/$evaluate-measure?measure={}&periodStart=2000&periodEnd=2030",
        BLAZE_BASE_URL.to_owned(),
        url
    ))
    .await
    .map_err(|e| SpotError::MeasureEvaluationError(e))?;

    if resp.status() == StatusCode::OK {
        println!(
            "Successfully evaluated the Measure with canonical URL: {}",
            url
        );
        text = resp.text().await.map_err(|e| SpotError::MeasureEvaluationError(e))?;
    } else {
        let error_message = format!(
            "Error while evaluating the Measure with canonical URL `{}`: {:?}",
            url, resp
        );
        println!("{}", error_message);
    }

    Ok(text)
}

pub async fn run_cql_query(library: Value, measure: Value) -> Result<String, SpotError> {
    let mut url:String = String::new();
    if let Ok(value) = util::get_json_field(&measure.to_string(), "url"){
        url = value.to_string().replace("\"", "");

    };
    println!("Evaluating the Measure with canonical URL: {}", url);

    let result_library = post_library(library.to_string()).await;
    match result_library {
        Err(err) => println!("Error: {}", err),
        Ok(()) => {
            let result_measure = post_measure(measure.to_string()).await;
            match result_measure {
                Err(err) => println!("Error: {}", err),
                Ok(()) => {
                    let result_evaluation = evaluate_measure(url).await;
                    match result_evaluation {
                        Err(err) => println!("Error: {}", err),
                        Ok(text) => {
                            return Ok(text);
                        }
                    }
                }
            }
        }
    }

    Err(SpotError::CQLQueryError())
}
