use http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use tracing::{debug, warn};

use crate::config::CONFIG;
use crate::errors::FocusError;
use crate::ast;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OmopQuery{
    pub lang: String,
    pub query: String
}



pub async fn post_ast(ast: ast::Ast) -> Result<String, FocusError> {
    debug!("Posting AST...");

    let ast_string = serde_json::to_string_pretty(&ast)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;

    debug!("{}", ast_string.clone());

    let resp = CONFIG
        .client
        .post(format!("{}", CONFIG.endpoint_url))
        .header("Content-Type", "application/json")
        .body(ast_string.clone())
        .send()
        .await
        .map_err(|e| FocusError::UnableToPostAst(e))?;

    debug!("Posted AST...");

    let text = match resp.status() {
        StatusCode::OK => {
            resp
            .text()
            .await
            .map_err(|e| FocusError::UnableToPostAst(e))?
        },
        code => {
            warn!("Got unexpected code {code} while posting AST; reply was `{}`, debug info: {:?}", ast_string, resp);
            return Err(FocusError::AstPostingErrorReqwest(format!(
                "Error while posting AST `{}`: {:?}",
                ast_string, resp
            )));
        }
    };

    Ok(text)
}
