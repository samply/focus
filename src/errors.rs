use thiserror::Error;

#[derive(Error, Debug)]
pub enum FocusError {
    #[error("Unable to post FHIR Library")]
    UnableToPostLibrary(reqwest::Error),
    #[error("Unable to post FHIR Measure")]
    UnableToPostMeasure(reqwest::Error),
    #[error("FHIR Measure evaluation error in Reqwest")]
    MeasureEvaluationErrorReqwest(reqwest::Error),
    #[error("FHIR Measure evaluation error in Blaze")]
    MeasureEvaluationErrorBlaze(String),
    #[error("CQL query error")]
    CQLQueryError(),
    #[error("Unable to retrieve tasks from Beam: {0}")]
    UnableToRetrieveTasksHttp(beam_lib::BeamError),
    #[error("Unable to answer task")]
    UnableToAnswerTask(beam_lib::BeamError),
    #[error("Unable to set proxy settings")]
    InvalidProxyConfig(reqwest::Error),
    #[error("Decode error")]
    DecodeError(base64::DecodeError),
    #[error("Configuration error")]
    ConfigurationError(String),
    #[error("Cannot open file")]
    FileOpeningError(String),
    #[error("Parsing error")]
    ParsingError(String),
    #[error("CQL tampered with: {0}")]
    CQLTemperedWithError(String),
    #[error("Laplace error: {0}")]
    LaplaceError(laplace_rs::errors::LaplaceError),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Unable to post AST: {0}")]
    UnableToPostAst(reqwest::Error),
    #[error("AST Posting error in Reqwest: {0}")]
    AstPostingErrorReqwest(String),
    #[error("Unknown criterion in AST: {0}")]
    AstUnknownCriterion(String),
    #[error("Unknown option in AST: {0}")]
    AstUnknownOption(String),
    #[error("Mismatch between operator and value type")]
    AstOperatorValueMismatch(),
    #[error("Invalid date format: {0}")]
    AstInvalidDateFormat(String),
}
