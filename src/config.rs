use std::path::PathBuf;

use http::{Uri, HeaderValue};
use reqwest::{Proxy, Client, Certificate};
use static_init::dynamic;
use tracing::{debug, info, warn};
use clap::Parser;

use crate::{errors::FocusError, beam::AppId};

#[dynamic(lazy)]
pub(crate) static CONFIG: Config = {
    debug!("Loading config");
    Config::load().unwrap_or_else(|e| {
        eprintln!("Unable to start as there was an error reading the config:\n{}\n\nTerminating -- please double-check your startup parameters with --help and refer to the documentation.", e);
        std::process::exit(1);
    })
};

const CLAP_FOOTER: &str = "For proxy support, environment variables HTTP_PROXY, HTTPS_PROXY, ALL_PROXY and NO_PROXY (and their lower-case variants) are supported. Usually, you want to set HTTP_PROXY *and* HTTPS_PROXY or set ALL_PROXY if both values are the same.\n\nFor updates and detailed usage instructions, visit https://github.com/samply/focus";

#[derive(Parser,Debug)]
#[clap(name("🔭 Focus"), version, arg_required_else_help(true), after_help(CLAP_FOOTER))]
struct CliArgs {
    /// The beam proxy's base URL, e.g. https://proxy1.beam.samply.de
    #[clap(long, env, value_parser)]
    beam_proxy_url: Uri,

    /// This application's beam AppId, e.g. focus.proxy1.broker.samply.de
    #[clap(long, env, value_parser)]
    beam_app_id: String,

    /// This applications beam API key
    #[clap(long, env, value_parser)]
    api_key: String,

    /// Number of retries for reaching the beam proxy and FHIR server, respectively
    #[clap(long, env, value_parser)]
    retry_count: usize,

    /// The FHIR servers base URL, e.g. https://blaze.site/fhir
    #[clap(long, env, value_parser)]
    blaze_url: Uri,

    /// Outgoing HTTP proxy: Directory with CA certificates to trust for TLS connections (e.g. /etc/samply/cacerts/)
    #[clap(long, env, value_parser)]
    tls_ca_certificates_dir: Option<PathBuf>,
}

pub(crate) struct Config {
    pub beam_proxy_url: Uri,
    pub beam_app_id: AppId,
    pub api_key: String,
    pub retry_count: usize,
    pub blaze_url: Uri,
    tls_ca_certificates: Vec<Certificate>,
    pub client: Client,
}

impl Config {
    fn load() -> Result<Self,FocusError> {
        let cli_args = CliArgs::parse(); 
        info!("Successfully read config and API keys from CLI and secrets files.");
        let tls_ca_certificates_dir = cli_args.tls_ca_certificates_dir;
        let tls_ca_certificates = load_certificates_from_dir(tls_ca_certificates_dir.clone())
            .map_err(|e| FocusError::ConfigurationError(format!("Unable to read from TLS CA directory: {}", e)))?;
        let client = prepare_reqwest_client(&tls_ca_certificates)?;
        let config = Config {
            beam_proxy_url: cli_args.beam_proxy_url,
            beam_app_id: AppId::new(cli_args.beam_app_id)?,
            api_key: cli_args.api_key,
            retry_count: cli_args.retry_count,
            blaze_url: cli_args.blaze_url,
            tls_ca_certificates,
            client
        };
        Ok(config)
    }
}


pub fn load_certificates_from_dir(ca_dir: Option<PathBuf>) -> Result<Vec<Certificate>, std::io::Error> {
    let mut result = Vec::new();
    if let Some(ca_dir) = ca_dir {
        for file in ca_dir.read_dir()? { //.map_err(|e| SamplyBeamError::ConfigurationFailed(format!("Unable to read from TLS CA directory {}: {}", ca_dir.to_string_lossy(), e)))
            let path = file?.path();
            let content = std::fs::read(&path)?;
            let cert = Certificate::from_pem(&content);
            if let Err(e) = cert {
                warn!("Unable to read certificate from file {}: {}", path.to_string_lossy(), e);
                continue;
            }
            result.push(cert.unwrap());
        }
    }
    Ok(result)
}

pub fn prepare_reqwest_client(certs: &Vec<Certificate>) -> Result<reqwest::Client, FocusError>{
    let mut client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .user_agent(HeaderValue::from_static(env!("SAMPLY_USER_AGENT")));
    for cert in certs {
        client = client.add_root_certificate(cert.to_owned());
    }
    let mut proxies: Vec<Proxy> = Vec::new();
    let no_proxy = reqwest::NoProxy::from_env();
    for var in ["http_proxy", "https_proxy", "all_proxy", "no_proxy"] {
        for (k,v) in std::env::vars().filter(|(k,_)| k.to_lowercase() == var) {
            std::env::set_var(k.to_uppercase(), v.clone());
            match k.as_str() {
                "http_proxy" => proxies.push(Proxy::http(v).map_err(|e|FocusError::InvalidProxyConfig(e))?.no_proxy(no_proxy.clone())),
                "https_proxy" => proxies.push(Proxy::https(v).map_err(|e|FocusError::InvalidProxyConfig(e))?.no_proxy(no_proxy.clone())),
                "all_proxy" => proxies.push(Proxy::all(v).map_err(|e|FocusError::InvalidProxyConfig(e))?.no_proxy(no_proxy.clone())),
                _ => ()
            };
        }
    }
    client.build().map_err(|e|FocusError::ConfigurationError(format!("Cannot create http client: {}",e)))

}
