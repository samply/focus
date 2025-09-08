use reqwest::header;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FocusError {
    #[error("Unable to post FHIR Library: {0}")]
    UnableToPostLibrary(reqwest::Error),
    #[error("Unable to post FHIR Measure: {0}")]
    UnableToPostMeasure(reqwest::Error),
    #[error("FHIR Measure evaluation error in Reqwest: {0}")]
    MeasureEvaluationErrorReqwest(reqwest::Error),
    #[error("FHIR Measure evaluation error in Blaze: {0}")]
    MeasureEvaluationErrorBlaze(String),
    #[error("CQL query error")]
    CQLQueryError,
    #[error("Unable to retrieve tasks from Beam: {0}")]
    UnableToRetrieveTasksHttp(beam_lib::BeamError),
    #[error("Unable to answer task: {0}")]
    UnableToAnswerTask(beam_lib::BeamError),
    #[error("Unable to set proxy settings: {0}")]
    InvalidProxyConfig(reqwest::Error),
    #[error("Decode error: {0}")]
    DecodeError(base64::DecodeError),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Cannot open file: {0}")]
    FileOpeningError(String),
    #[error("Serde parsing error: {0}")]
    SerdeParsingError(#[from] serde_json::Error),
    #[error("Parsing error: {0}")]
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
    #[error("Unable to post EUCAIM API query: {0}")]
    UnableToPostEucaimApiQuery(reqwest::Error),
    #[error("EUCAIM query generation error")]
    EucaimQueryGenerationError,
    #[error("Unable to post Exporter query: {0}")]
    UnableToPostExporterQuery(reqwest::Error),
    #[error("Unable to get Exporter query status: {0}")]
    UnableToGetExporterQueryStatus(reqwest::Error),
    #[error("Exporter query error in Reqwest: {0}")]
    ExporterQueryErrorReqwest(String),
    #[error("AST Posting error in Reqwest: {0}")]
    AstPostingErrorReqwest(String),
    #[error("Unknown criterion in AST: {0}")]
    AstUnknownCriterion(String),
    #[error("Unknown option in AST: {0}")]
    AstUnknownOption(String),
    #[error("Mismatch between operator and value type: {0}")]
    AstOperatorValueMismatch(String),
    #[error("Invalid date format: {0}")]
    AstInvalidDateFormat(String),
    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(header::InvalidHeaderValue),
    #[error("Missing Exporter Endpoint")]
    MissingExporterEndpoint,
    #[error("Missing Exporter Task Type")]
    MissingExporterTaskType,
    #[error("Cannot connect to database: {0}")]
    CannotConnectToDatabase(String),
    #[error("QueryResultBad: {0}")]
    QueryResultBad(String),
    #[error("Query not allowed: {0}")]
    QueryNotAllowed(String),
    #[cfg(feature = "query-sql")]
    #[error("Error executing SQL query: {0}")]
    ErrorExecutingSqlQuery(sqlx::Error),
}

impl FocusError {
    /// Generate a descriptive error message that does not leak any sensitive data that might be contained inside the error value
    pub fn user_facing_error(&self) -> &'static str {
        use FocusError::*;
        // TODO: Add more match arms
        match self {
            DecodeError(_) | ParsingError(_) | SerdeParsingError(_) => "Cannot parse query.",
            LaplaceError(_) => "Cannot obfuscate result.",
            _ => "Failed to execute query.",
        }
    }
}
