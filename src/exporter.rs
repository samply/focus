use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::config::CONFIG;
use crate::errors::FocusError;

struct Words {
    method: &'static str,
    doing: &'static str,
    done: &'static str,
}

static WORDS: Lazy<HashMap<bool, Words>> = Lazy::new(|| {
    [
        (
            true,
            Words {
                method: "request",
                doing: "executing",
                done: "executed",
            },
        ),
        (
            false,
            Words {
                method: "create-query",
                doing: "creating",
                done: "created",
            },
        ),
    ]
    .into()
});

pub async fn post_exporter_query(body: &String, execute: bool) -> Result<String, FocusError> {
    match &CONFIG.exporter_url {
        None => Err(FocusError::MissingExporterEndpoint()),
        Some(exporter_url) => {
            debug!("{} exporter query...", WORDS.get(&execute).unwrap().doing);

            let mut headers = HeaderMap::new();

            headers.insert(
                header::CONTENT_TYPE, //TODO discard the result, just return OK
                HeaderValue::from_str("text/html; charset=UTF-8")
                    .map_err(FocusError::InvalidHeaderValue)?,
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
                    WORDS.get(&execute).unwrap().method
                ))
                .headers(headers)
                .body(body.clone())
                .send()
                .await
                .map_err(FocusError::UnableToPostExporterQuery)?;

            debug!("{} query...", WORDS.get(&execute).unwrap().done);

            let text = match resp.status() {
                StatusCode::OK => {
                    format!("Query successfully {}", WORDS.get(&execute).unwrap().done)
                }
                code => {
                    warn!("Got unexpected code {code} while {} query; reply was `{}`, debug info: {:?}", WORDS.get(&execute).unwrap().doing, body, resp);
                    return Err(FocusError::ExporterQueryErrorReqwest(format!(
                        "Error while {} query: {:?}",
                        WORDS.get(&execute).unwrap().doing,
                        resp
                    )));
                }
            };

            Ok(text)
        }
    }
}
