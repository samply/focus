use thiserror::Error;

#[derive(Error, Debug)]
pub enum FocusError {
    #[error("Unable to post FHIR Library")]
    UnableToPostLibrary(reqwest::Error),
    #[error("Unable to post FHIR Measure")]
    UnableToPostMeasure(reqwest::Error),
    #[error("FHIR Measure evaluation error")]
    MeasureEvaluationError(reqwest::Error),
    #[error("CQL query error")]
    CQLQueryError(),
    #[error("Unable to retrieve tasks from Beam: {0}")]
    UnableToRetrieveTasksHttp(reqwest::Error),
    #[error("Unable to retrieve tasks from Beam: {0}")]
    UnableToRetrieveTasksOther(String),
    #[error("Unable to parse tasks from Beam")]
    UnableToParseTasks(reqwest::Error),
    #[error("Unable to answer task")]
    UnableToAnswerTask(reqwest::Error),
    #[error("Unable to set proxy settings")]
    InvalidProxyConfig(reqwest::Error),
    #[error("Decode error")]
    DecodeError(base64::DecodeError),
    #[error("Parse error")]
    ParseError(serde_json::Error),
    #[error("Configuration error")]
    ConfigurationError(String),
    #[error("Invalid BeamID")]
    InvalidBeamId(String),
    #[error("Parsing error")]
    ParsingError(String),
    #[error("CQL tempered with")]
    CQLTemperedWithError(String),
}
