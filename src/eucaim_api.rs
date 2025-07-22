use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode, Url,
};
use tracing::{debug, error, trace, warn};

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::ast;
use crate::config::CONFIG;
use crate::errors::FocusError;

pub static CATEGORY: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert("SNOMEDCT263495000", "gender");
    map.insert("SNOMEDCT439401001", "diagnosis");
    map.insert("RID10311", "modality");
    map.insert("SNOMEDCT123037004", "bodyPart");
    map.insert("C25392", "manufacturer");

    map
});

pub static CRITERION: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert("SNOMEDCT248153007", "male");
    map.insert("SNOMEDCT248152002", "female");
    map.insert("SNOMEDCT74964007", "other");
    map.insert("SNOMEDCT261665006", "unknown");
    map.insert("SNOMEDCT363406005", "SNOMEDCT363406005"); // colon cancer
    map.insert("SNOMEDCT254837009", "SNOMEDCT254837009"); // breast cancer
    map.insert("SNOMEDCT363358000", "SNOMEDCT363358000"); // lung cancer
    map.insert("SNOMEDCT363484005", "SNOMEDCT363484005"); // pelvis cancer
    map.insert("SNOMEDCT399068003", "SNOMEDCT399068003"); // prostate cancer
    map.insert("RID10312", "MR");
    map.insert("RID10337", "PET");
    map.insert("RID10334", "SPECT");
    map.insert("RID10321", "CT");
    map.insert("SNOMEDCT76752008", "breast");
    map.insert("SNOMEDCT71854001", "colon");
    map.insert("SNOMEDCT39607008", "lung");
    map.insert("SNOMEDCT12921003", "pelvis");
    map.insert("SNOMEDCT41216001", "prostate");
    map.insert("C200140", "Siemens");
    map.insert("birnlex_3066", "Siemens");
    map.insert("birnlex_12833", "General%20Electric");
    map.insert("birnlex_3065", "Philips");
    map.insert("birnlex_3067", "Toshiba");

    map
});

pub fn build_eucaim_api_query_url(base_url: Url, ast: ast::Ast) -> Result<String, FocusError> {
    let mut url: String = base_url.to_string() + "?";

    let mut parameters: Vec<String> = Vec::new();

    let children = ast.ast.children;

    if children.len() > 1 {
        error!("Too many children! AND/OR queries not supported.");
        return Err(FocusError::EucaimApiQueryGenerationError);
    }

    for child in children {
        // will be either 0 or 1
        match child {
            ast::Child::Operation(operation) => {
                if operation.operand == ast::Operand::Or {
                    error!("OR found as first level operator");
                    return Err(FocusError::EucaimApiQueryGenerationError);
                }
                for grandchild in operation.children {
                    match grandchild {
                        ast::Child::Operation(operation) => {
                            if operation.operand == ast::Operand::And {
                                error!("AND found as second level operator");
                                return Err(FocusError::EucaimApiQueryGenerationError);
                            }
                            let greatgrandchildren = operation.children;
                            if greatgrandchildren.len() > 1 {
                                error!("Too many children! OR operator between criteria of the same type not supported.");
                                return Err(FocusError::EucaimApiQueryGenerationError);
                            }

                            for greatgrandchild in greatgrandchildren {
                                match greatgrandchild {
                                    ast::Child::Operation(_) => {
                                        error!(
                                            "Search tree has too many levels. Query not supported"
                                        );
                                        return Err(FocusError::EucaimApiQueryGenerationError);
                                    }
                                    ast::Child::Condition(condition) => {
                                        let category = CATEGORY.get(&(condition.key).as_str());
                                        if let Some(cat) = category {
                                            match condition.value {
                                                ast::ConditionValue::String(value) => {
                                                    let criterion =
                                                        CRITERION.get(&(value).as_str());
                                                    if let Some(crit) = criterion {
                                                        parameters
                                                            .push(cat.to_string() + "=" + crit);
                                                    }
                                                }
                                                _ => {
                                                    error!("The only supported condition value type is string");
                                                    return Err(
                                                        FocusError::EucaimApiQueryGenerationError,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ast::Child::Condition(_) => {
                            // must be operation
                            error!("Condition found as second level child");
                            return Err(FocusError::EucaimApiQueryGenerationError);
                        }
                    }
                }
            }
            ast::Child::Condition(_) => {
                // must be operation
                error!("Condition found as first level child");
                return Err(FocusError::EucaimApiQueryGenerationError);
            }
        }
    }

    url += parameters.join("&").as_str();

    trace!("{}", &url);

    Ok(url)
}

pub async fn send_eucaim_api_query(ast: ast::Ast) -> Result<String, FocusError> {
    debug!("Posting EUCAIM API query...");

    let eucaim_api_query =
        if let Ok(query) = build_eucaim_api_query_url(CONFIG.endpoint_url.clone(), ast) {
            query
        } else {
            return Err(FocusError::EucaimApiQueryGenerationError);
        };

    let mut headers = HeaderMap::new();

    if let Some(auth_header_value) = CONFIG.auth_header.clone() {
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(auth_header_value.as_str())
                .map_err(FocusError::InvalidHeaderValue)?,
        );
    }

    let resp = CONFIG
        .client
        .get(&eucaim_api_query)
        .headers(headers)
        .send()
        .await
        .map_err(FocusError::UnableToPostEucaimApiQuery)?;

    debug!("Posted EUCAIM API query...");

    let text = match resp.status() {
        StatusCode::OK => resp.text().await.map_err(FocusError::UnableToPostAst)?,
        code => {
            warn!(
                "Got unexpected code {code} while posting EUCAIM API query; reply was `{}`, debug info: {:?}",
                eucaim_api_query, resp
            );
            return Err(FocusError::AstPostingErrorReqwest(format!(
                "Error while posting AST `{}`: {:?}",
                eucaim_api_query, resp
            )));
        }
    };

    Ok(text)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions;

    const EMPTY: &str = r#"{"ast":{"children":[],"operand":"OR"},"id":"ef8bae78-522c-498c-b7db-3f96f279a1a0__search__ef8bae78-522c-498c-b7db-3f96f279a1a0"}"#;

    const JUST_RIGHT: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248153007"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT399068003"}],"operand":"OR"},{"children":[{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10312"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT76752008"}],"operand":"OR"},{"children":[{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3065"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"66b8bbf4-ded2-4f94-87ab-3a3ca2f4edc0__search__66b8bbf4-ded2-4f94-87ab-3a3ca2f4edc0"}"#;

    const TOO_MUCH: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248153007"},{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248152002"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT399068003"},{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT254837009"}],"operand":"OR"},{"children":[{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10312"},{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10337"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT76752008"},{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT41216001"}],"operand":"OR"},{"children":[{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3065"},{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3067"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"c57e075c-19de-4c5a-ba9c-b8f697a98dfc__search__c57e075c-19de-4c5a-ba9c-b8f697a98dfc"}"#;

    #[test]
    fn test_build_url_empty() {
        let url = build_eucaim_api_query_url(
            Url::parse("http://base.info/search").unwrap(),
            serde_json::from_str(EMPTY).unwrap(),
        )
        .unwrap();
        pretty_assertions::assert_eq!(url, "http://base.info/search?");
    }

    #[test]
    fn test_build_url_just_right() {
        let url = build_eucaim_api_query_url(
            Url::parse("http://base.info/search").unwrap(),
            serde_json::from_str(JUST_RIGHT).unwrap(),
        )
        .unwrap();
        pretty_assertions::assert_eq!(url, "http://base.info/search?gender=male&diagnosis=SNOMEDCT399068003&modality=MR&bodyPart=breast&manufacturer=Philips");
    }

    #[test]
    fn test_build_url_too_much() {
        assert!(build_eucaim_api_query_url(
            Url::parse("http://base.info/search").unwrap(),
            serde_json::from_str(TOO_MUCH).unwrap()
        )
        .is_err());
    }
}
