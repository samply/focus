use http::header;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, warn};

use crate::ast;
use crate::config::CONFIG;
use crate::errors::FocusError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OmopQuery {
    pub lang: String,
    pub query: String,
}

pub async fn post_ast(ast: ast::Ast) -> Result<String, FocusError> {
    debug!("Posting AST...");

    let ast_string = serde_json::to_string_pretty(&ast)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;

    debug!("{}", ast_string.clone());

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("application/json").map_err(FocusError::InvalidHeaderValue)?,
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
        .post(format!("{}", CONFIG.endpoint_url))
        .headers(headers)
        .body(ast_string.clone())
        .send()
        .await
        .map_err(FocusError::UnableToPostAst)?;

    debug!("Posted AST...");

    let text = match resp.status() {
        StatusCode::OK => resp
            .text()
            .await
            .map_err(FocusError::UnableToPostAst)?,
        code => {
            warn!(
                "Got unexpected code {code} while posting AST; reply was `{}`, debug info: {:?}",
                ast_string, resp
            );
            return Err(FocusError::AstPostingErrorReqwest(format!(
                "Error while posting AST `{}`: {:?}",
                ast_string, resp
            )));
        }
    };

    Ok(text)
}
