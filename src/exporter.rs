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
    match &CONFIG.exporter_url {
        None => Err(FocusError::MissingExporterEndpoint()),
        Some(exporter_url) => {
            let exporter_params = if execute { EXECUTE } else { CREATE };
            debug!("{} exporter query...", exporter_params.doing);

            let mut headers = HeaderMap::new();

            headers.insert(
                header::CONTENT_TYPE, //TODO discard the result, just return OK
                HeaderValue::from_static("text/html; charset=UTF-8"),
            );

            if let Some(auth_header_value) = CONFIG.auth_header.clone() {
                headers.insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(auth_header_value.as_str())
                        .map_err(FocusError::InvalidHeaderValue)?,
                );
            }

            let resp = CONFIG
                .client
                .post(format!(
                    "{}{}",
                    exporter_url,
                    exporter_params.method
                ))
                .headers(headers)
                .body(body.clone())
                .send()
                .await
                .map_err(FocusError::UnableToPostExporterQuery)?;

            debug!("{} query...", exporter_params.done);

            let text = match resp.status() {
                StatusCode::OK => {
                    format!("Query successfully {}", exporter_params.done)
                }
                code => {
                    warn!("Got unexpected code {code} while {} query; reply was `{}`, debug info: {:?}", exporter_params.doing, body, resp);
                    return Err(FocusError::ExporterQueryErrorReqwest(format!(
                        "Error while {} query: {:?}",
                        exporter_params.doing,
                        resp
                    )));
                }
            };

            Ok(text)
        }
    }
}
