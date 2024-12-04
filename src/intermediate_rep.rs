use reqwest::{header::{self, HeaderMap, HeaderValue}, StatusCode};
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, warn};

use crate::{ast, config::FocusBackend};
use crate::config::CONFIG;
use crate::errors::FocusError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntermediateRepQuery {
    pub lang: String,
    pub query: String,
}

pub(crate) struct IntermediateRep;

impl FocusBackend for IntermediateRep {
    fn make_authheader(apikey: &str) -> Result<(header::HeaderName, HeaderValue), FocusError> {
        let name = header::AUTHORIZATION;
        let value = HeaderValue::from_str(apikey)
            .map_err(|e| FocusError::ConfigurationError(format!("Invalid value \"{}\" in apikey for ast2sql backend: {}", apikey, e)))?;
        Ok((name, value))
    }

    async fn check_availability() -> bool {
        // TODO: Implement
        true
    }
}

pub async fn post_ast(ast: ast::Ast) -> Result<String, FocusError> {
    debug!("Posting AST...");

    let ast_string = serde_json::to_string_pretty(&ast)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    if let Some(auth_header) = CONFIG.backend_ast2sql_authheader.clone() {
        headers.insert(auth_header.0, auth_header.1);
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
        StatusCode::OK => resp.text().await.map_err(FocusError::UnableToPostAst)?,
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
