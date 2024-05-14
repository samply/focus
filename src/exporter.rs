use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use tracing::{debug, warn};

use crate::config::CONFIG;
use crate::errors::FocusError;

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

pub async fn post_exporter_query(body: &String, execute: bool) -> Result<String, FocusError> {
    let Some(exporter_url) = &CONFIG.exporter_url else {
        return Err(FocusError::MissingExporterEndpoint());
    };

    let exporter_params = if execute { EXECUTE } else { CREATE };
    debug!("{} exporter query...", exporter_params.doing);

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE, 
        HeaderValue::from_static("application/json"),
    );

    if let Some(auth_header_value) = CONFIG.auth_header.clone() {
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(auth_header_value.as_str())
                .map_err(FocusError::InvalidHeaderValue)?,
        );
    }

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
