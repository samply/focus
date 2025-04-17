use std::fmt;
use std::path::PathBuf;

use beam_lib::AppId;
use clap::Parser;
use reqwest::{header::HeaderValue, Url};
use once_cell::sync::Lazy;
use reqwest::{Certificate, Client, Proxy};
use tracing::{debug, info, warn};

use crate::errors::FocusError;

#[derive(clap::ValueEnum, Clone, PartialEq, Debug)]
pub enum Obfuscate {
    No,
    Yes,
}

#[derive(clap::ValueEnum, Clone, PartialEq, Debug, Copy)]
pub enum EndpointType {
    Blaze,
    Omop, // endpoint is URL of a query mediator translating AST to provider specific SQL
    EucaimApi, // endpoint is URL of custom API for querying EUCAIM provider
    #[cfg(feature = "query-sql")]
    BlazeAndSql,
    #[cfg(feature = "query-sql")]
    Sql,
}

impl fmt::Display for EndpointType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndpointType::Blaze => write!(f, "blaze"),
            EndpointType::Omop => write!(f, "omop"),
            EndpointType::EucaimApi => write!(f, "eucaim_api"),
            #[cfg(feature = "query-sql")]
            EndpointType::BlazeAndSql => write!(f, "blaze_and_sql"),
            #[cfg(feature = "query-sql")]
            EndpointType::Sql => write!(f, "sql"),
        }
    }
}

pub(crate) static CONFIG: Lazy<Config> = Lazy::new(|| {
    debug!("Loading config");
    Config::load().unwrap_or_else(|e| {
        eprintln!("Unable to start as there was an error reading the config:\n{}\n\nTerminating -- please double-check your startup parameters with --help and refer to the documentation.", e);
        std::process::exit(1);
    })
});

const CLAP_FOOTER: &str = "For proxy support, environment variables HTTP_PROXY, HTTPS_PROXY, ALL_PROXY and NO_PROXY (and their lower-case variants) are supported. Usually, you want to set HTTP_PROXY *and* HTTPS_PROXY or set ALL_PROXY if both values are the same.\n\nFor updates and detailed usage instructions, visit https://github.com/samply/focus";

#[derive(Parser, Debug)]
#[clap(
    name("🔭 Focus"),
    version,
    arg_required_else_help(true),
    after_help(CLAP_FOOTER)
)]
struct CliArgs {
    /// The beam proxy's base URL, e.g. https://proxy1.beam.samply.de
    #[clap(long, env, value_parser)]
    beam_proxy_url: Url,

    /// This application's beam AppId, e.g. focus.proxy1.broker.samply.de
    #[clap(long, env, value_parser)]
    beam_app_id_long: String,

    /// This applications beam API key
    #[clap(long, env, value_parser)]
    api_key: String,

    /// Number of retries for reaching the beam proxy and FHIR server, respectively
    #[clap(long, env, value_parser, default_value = "32")]
    retry_count: usize,

    /// The endpoint base URL, e.g. https://blaze.site/fhir/
    #[clap(long, env, value_parser)]
    endpoint_url: Option<Url>,

    /// The endpoint base URL, e.g. https://blaze.site/fhir/, for the sake of backward compatibility, use endpoint_url instead
    #[clap(long, env, value_parser)]
    blaze_url: Option<Url>,

    /// The exporter URL, e.g. https://exporter.site/
    #[clap(long, env, value_parser)]
    exporter_url: Option<Url>,

    /// Type of the endpoint, e.g. "blaze", "omop"
    #[clap(long, env, value_parser = clap::value_parser!(EndpointType), default_value = "blaze")]
    endpoint_type: EndpointType,

    /// Should the results be obfuscated
    #[clap(long, env, value_parser = clap::value_parser!(Obfuscate), default_value = "yes")]
    obfuscate: Obfuscate,

    /// Should zero values be obfuscated - default false
    #[clap(long, env, value_parser)]
    obfuscate_zero: bool,

    /// The mode of obfuscating values below 10: 0 - return zero, 1 - return ten, 2 - obfuscate using Laplace distribution and rounding
    #[clap(long, env, value_parser, default_value = "1")]
    obfuscate_below_10_mode: usize,

    /// Sensitivity parameter for obfuscating the counts in the Patient stratifier
    #[clap(long, env, value_parser, default_value = "1.")]
    delta_patient: f64,

    /// Sensitivity parameter for obfuscating the counts in the Specimen stratifier
    #[clap(long, env, value_parser, default_value = "20.")]
    delta_specimen: f64,

    /// Sensitivity parameter for obfuscating the counts in the Diagnosis stratifier
    #[clap(long, env, value_parser, default_value = "3.")]
    delta_diagnosis: f64,

    /// Sensitivity parameter for obfuscating the counts in the Procedure stratifier
    #[clap(long, env, value_parser, default_value = "1.7")]
    delta_procedures: f64,

    /// Sensitivity parameter for obfuscating the counts in the Medication Statements stratifier
    #[clap(long, env, value_parser, default_value = "2.1")]
    delta_medication_statements: f64,

    /// Sensitivity parameter for obfuscating the counts in the Histo stratifier
    #[clap(long, env, value_parser, default_value = "20.")]
    delta_histo: f64,

    /// Privacy budget parameter for obfuscating the counts in the stratifiers
    #[clap(long, env, value_parser, default_value = "0.1")]
    epsilon: f64,

    /// The granularity of the rounding of the obfuscated values
    #[clap(long, env, value_parser, default_value = "10")]
    rounding_step: usize,

    /// Projects for which the results are not to be obfuscated, separated by ;
    #[clap(
        long,
        env,
        value_parser,
        default_value = "exliquid;dktk_supervisors;exporter;ehds2"
    )]
    projects_no_obfuscation: String,

    ///The path to a file containing base64 encoded CQL queries, and aliases of SQL queries, whose results are to be cached. If not set, no results are cached
    #[clap(long, env, value_parser)]
    queries_to_cache: Option<PathBuf>,

    /// Outgoing HTTP proxy: Directory with CA certificates to trust for TLS connections (e.g. /etc/samply/cacerts/)
    #[clap(long, env, value_parser)]
    tls_ca_certificates_dir: Option<PathBuf>,

    /// OMOP provider name
    #[clap(long, env, value_parser)]
    provider: Option<String>,

    /// Base64 encoded OMOP provider icon
    #[clap(long, env, value_parser)]
    provider_icon: Option<String>,

    // TODO - refactor to include multiple authorization headers for multiple stores / applications at the same time
    /// Authorization header
    #[clap(long, env, value_parser)]
    auth_header: Option<String>,

    /// Exporter API key
    #[clap(long, env, value_parser)]
    exporter_api_key: Option<String>,

    /// Postgres connection string
    #[cfg(feature = "query-sql")]
    #[clap(long, env, value_parser)]
    postgres_connection_string: Option<String>,

    /// Max number of attempts to connect to the database
    #[cfg(feature = "query-sql")]
    #[clap(long, env, value_parser, default_value = "8")]
    max_db_attempts: u32,
}

pub(crate) struct Config {
    pub beam_proxy_url: Url,
    pub beam_app_id_long: AppId,
    pub api_key: String,
    pub retry_count: usize,
    pub endpoint_url: Url,
    pub exporter_url: Option<Url>,
    pub endpoint_type: EndpointType,
    pub obfuscate: Obfuscate,
    pub obfuscate_zero: bool,
    pub obfuscate_below_10_mode: usize,
    pub delta_patient: f64,
    pub delta_specimen: f64,
    pub delta_diagnosis: f64,
    pub delta_procedures: f64,
    pub delta_medication_statements: f64,
    pub delta_histo: f64,
    pub epsilon: f64,
    pub rounding_step: usize,
    pub unobfuscated: Vec<String>,
    pub queries_to_cache: Option<PathBuf>,
    pub client: Client,
    pub provider: Option<String>,
    pub provider_icon: Option<String>,
    pub auth_header: Option<String>,
    pub exporter_api_key: Option<String>,
    #[cfg(feature = "query-sql")]
    pub postgres_connection_string: Option<String>,
    #[cfg(feature = "query-sql")]
    pub max_db_attempts: u32,
}

impl Config {
    fn load() -> Result<Self, FocusError> {
        let cli_args = CliArgs::parse();
        info!("Successfully read config and API keys from CLI and secrets files.");
        let tls_ca_certificates_dir = cli_args.tls_ca_certificates_dir;
        let tls_ca_certificates = load_certificates_from_dir(tls_ca_certificates_dir.clone())
            .map_err(|e| {
                FocusError::ConfigurationError(format!(
                    "Unable to read from TLS CA directory: {}",
                    e
                ))
            })?;
        let client = prepare_reqwest_client(&tls_ca_certificates)?;
        dbg!(cli_args.endpoint_url.clone());
        dbg!(cli_args.blaze_url.clone());
        let config = Config {
            beam_proxy_url: cli_args.beam_proxy_url,
            beam_app_id_long: AppId::new_unchecked(cli_args.beam_app_id_long),
            api_key: cli_args.api_key,
            retry_count: cli_args.retry_count,
            endpoint_url: cli_args.endpoint_url.unwrap_or_else(|| cli_args.blaze_url.expect("Look, mate, you need to set endpoint-url or blaze-url, can't work without, sry")),
            exporter_url: cli_args.exporter_url,
            endpoint_type: cli_args.endpoint_type,
            obfuscate: cli_args.obfuscate,
            obfuscate_zero: cli_args.obfuscate_zero,
            obfuscate_below_10_mode: cli_args.obfuscate_below_10_mode,
            delta_patient: cli_args.delta_patient,
            delta_specimen: cli_args.delta_specimen,
            delta_diagnosis: cli_args.delta_diagnosis,
            delta_procedures: cli_args.delta_procedures,
            delta_medication_statements: cli_args.delta_medication_statements,
            delta_histo: cli_args.delta_histo,
            epsilon: cli_args.epsilon,
            rounding_step: cli_args.rounding_step,
            unobfuscated: cli_args.projects_no_obfuscation.split(';').map(|s| s.to_string()).collect(),
            queries_to_cache: cli_args.queries_to_cache,
            provider: cli_args.provider,
            provider_icon: cli_args.provider_icon,
            auth_header: cli_args.auth_header,
            exporter_api_key: cli_args.exporter_api_key,
            #[cfg(feature = "query-sql")]
            postgres_connection_string: cli_args.postgres_connection_string,
            #[cfg(feature = "query-sql")]
            max_db_attempts: cli_args.max_db_attempts,
            client,
        };
        Ok(config)
    }
}

pub fn load_certificates_from_dir(
    ca_dir: Option<PathBuf>,
) -> Result<Vec<Certificate>, std::io::Error> {
    let mut result = Vec::new();
    if let Some(ca_dir) = ca_dir {
        for file in ca_dir.read_dir()? {
            //.map_err(|e| SamplyBeamError::ConfigurationFailed(format!("Unable to read from TLS CA directory {}: {}", ca_dir.to_string_lossy(), e)))
            let path = file?.path();
            let content = std::fs::read(&path)?;
            let cert = Certificate::from_pem(&content);
            if let Err(e) = cert {
                warn!(
                    "Unable to read certificate from file {}: {}",
                    path.to_string_lossy(),
                    e
                );
                continue;
            }
            result.push(cert.unwrap());
        }
    }
    Ok(result)
}

pub fn prepare_reqwest_client(certs: &Vec<Certificate>) -> Result<reqwest::Client, FocusError> {
    let mut client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .user_agent(HeaderValue::from_static(env!("SAMPLY_USER_AGENT")));
    for cert in certs {
        client = client.add_root_certificate(cert.to_owned());
    }
    let mut proxies: Vec<Proxy> = Vec::new();
    let no_proxy = reqwest::NoProxy::from_env();
    for var in ["http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
        for (k, v) in std::env::vars().filter(|(k, _)| k.to_lowercase() == var) {
            std::env::set_var(k.to_uppercase(), v.clone());
            match k.as_str() {
                "http_proxy" => proxies.push(
                    Proxy::http(v)
                        .map_err(FocusError::InvalidProxyConfig)?
                        .no_proxy(no_proxy.clone()),
                ),
                "https_proxy" => proxies.push(
                    Proxy::https(v)
                        .map_err(FocusError::InvalidProxyConfig)?
                        .no_proxy(no_proxy.clone()),
                ),
                "all_proxy" => proxies.push(
                    Proxy::all(v)
                        .map_err(FocusError::InvalidProxyConfig)?
                        .no_proxy(no_proxy.clone()),
                ),
                _ => (),
            };
        }
    }
    client
        .build()
        .map_err(|e| FocusError::ConfigurationError(format!("Cannot create http client: {}", e)))
}
