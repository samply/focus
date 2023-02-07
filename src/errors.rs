use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpotError {
    #[error("Unable to post FHIR Library")]
    UnableToPostLibrary(reqwest::Error),
    #[error("Unable to post FHIR Measure")]
    UnableToPostMeasure(reqwest::Error),
    #[error("FHIR Measure evaluation error")]
    MeasureEvaluationError(reqwest::Error),
    #[error("CQL query error")]
    CQLQueryError(),
    #[error("Unable to retrieve tasks from Beam")]
    UnableToRetrieveTasks(reqwest::Error),
    #[error("Unable to parse tasks from Beam")]
    UnableToParseTasks(reqwest::Error),
    #[error("Unable to answer task")]
    UnableToAnswerTask(reqwest::Error),
    #[error("Unable to set proxy settings")]
    InvalidProxyConfig(reqwest::Error),
    #[error("Decode error")]
    DecodeError(base64::DecodeError),
    #[error("Configuration error")]
    ConfigurationError(String),
    #[error("Invalid BeamID")]
    InvalidBeamId(String),
    #[error("Parsing error")]
    ParsingError(String),
}