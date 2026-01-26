use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use tracing::{debug, error, trace, warn};

use crate::util::get_json_field;

use crate::ast;
use crate::config::CONFIG;
use crate::errors::FocusError;

pub static CATEGORY: Lazy<HashMap<&str, (&str, &str)>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, (&'static str, &'static str)> = HashMap::new();
    map.insert("SNOMEDCT263495000", ("sex", "patients"));
    map.insert("SNOMEDCT439401001", ("diagnosis", "patients"));
    map.insert("RID10311", ("imageModality", "imageStudies"));
    map.insert("SNOMEDCT123037004", ("imageBodyPart", "imageStudies"));
    map.insert("C25392", ("imageManufacturer", "imageStudies"));

    map
});

pub static CRITERION: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert("SNOMEDCT248153007", "EUCAIM:COM1001366"); //male
    map.insert("SNOMEDCT248152002", "EUCAIM:COM1001370"); //female
    map.insert("SNOMEDCT74964007", "EUCAIM:COM1001288"); //sex unspecified
    map.insert("SNOMEDCT261665006", "EUCAIM:COM1002760"); //sex unknown
    map.insert("SNOMEDCT363406005", "EUCAIM:CLIN1000057"); // colon cancer
    map.insert("SNOMEDCT254837009", "EUCAIM:CLIN1000060"); // breast cancer
    map.insert("SNOMEDCT363358000", "SNOMEDCT363358000"); // lung cancer
    map.insert("SNOMEDCT363484005", "SNOMEDCT363484005"); // pelvis cancer
    map.insert("SNOMEDCT399068003", "EUCAIM:CLIN1000075"); // prostate cancer
    map.insert("RID10312", "MR");
    map.insert("RID10337", "PET");
    map.insert("RID10334", "SPECT");
    map.insert("RID10321", "EUCAIM:CLIN1000185");
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

pub fn build_eucaim_beacon_body(ast: ast::Ast) -> Result<String, FocusError> {
    let mut body = String::from(r#"{
    "meta": {
        "apiVersion": "2.0"
    },
    "query":{ 
        "filters": ["#);
    let after_filter: String = String::from(r#"],
        "includeResultsetResponses": "HIT",
        "pagination": {
            "skip": 0,
            "limit": 10
        },
        "testMode": false,
        "requestedGranularity": "record"
    }
}"#);

    let mut parameters: Vec<String> = Vec::new();

    let children = ast.ast.children;

    if children.len() > 1 {
        error!("Too many children! OR queries not supported yet.");
        return Err(FocusError::EucaimQueryGenerationError);
    }

    for child in children {
        // will be either 0 or 1
        match child {
            ast::Child::Operation(operation) => {
                if operation.operand == ast::Operand::Or {
                    error!("OR found as first level operator");
                    return Err(FocusError::EucaimQueryGenerationError);
                }
                for grandchild in operation.children {
                    match grandchild {
                        ast::Child::Operation(operation) => {
                            if operation.operand == ast::Operand::And {
                                error!("AND found as second level operator");
                                return Err(FocusError::EucaimQueryGenerationError);
                            }
                            let greatgrandchildren = operation.children;
                            if greatgrandchildren.len() > 1 {
                                error!("Too many children! OR operator between criteria of the same type not supported.");
                                return Err(FocusError::EucaimQueryGenerationError);
                            }

                            for greatgrandchild in greatgrandchildren {
                                match greatgrandchild {
                                    ast::Child::Operation(_) => {
                                        error!(
                                            "Search tree has too many levels. Query not supported"
                                        );
                                        return Err(FocusError::EucaimQueryGenerationError);
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
                                                            //.push(cat.0.to_string() + "=" + crit);
                                                            .push(format!(r#"{{"id":"{}", "scope":"{}" }}"#, crit, cat.1.to_string()));
                                                    }
                                                }
                                                _ => {
                                                    error!("The only supported condition value type is string");
                                                    return Err(
                                                        FocusError::EucaimQueryGenerationError,
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
                            return Err(FocusError::EucaimQueryGenerationError);
                        }
                    }
                }
            }
            ast::Child::Condition(_) => {
                // must be operation
                error!("Condition found as first level child");
                return Err(FocusError::EucaimQueryGenerationError);
            }
        }
    }

    body += parameters.join(",").as_str();
    body += &after_filter;

    trace!("body: {}", &body);

    Ok(body)
}

pub async fn post_beacon_query(ast: ast::Ast) -> Result<String, FocusError> {
    debug!("Posting Beacon query...");

    let ast_string = serde_json::to_string_pretty(&ast)
        .map_err(|e| FocusError::SerializationError(e.to_string()))?;

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
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
        .post(format!("{}/collections", CONFIG.endpoint_url))
        .headers(headers)
        .body("")
        .send()
        .await
        .map_err(FocusError::UnableToQueryBeacon)?;

    debug!("Querying beacon...");

    let text = match resp.status() {
        StatusCode::OK => resp.text().await.map_err(FocusError::UnableToQueryBeacon)?,
        code => {
            warn!(
                "Got unexpected code {code} while querying Beacon; reply was `{:?}`, debug info: {}",
                resp, ast_string
            );
            return Err(FocusError::BeaconQueryingErrorReqwest(format!(
                "Error while querying Beacon `{}`: {:?}",
                ast_string, resp
            )));
        }
    };

    let response = get_json_field(&text, "response")?;

    Ok(serde_json::to_string(&response)?)
}
