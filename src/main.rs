
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let config = focus::Config::load().unwrap_or_else(|e| {
        eprintln!("Unable to start as there was an error reading the config:\n{}\n\nTerminating -- please double-check your startup parameters with --help and refer to the documentation.", e);
        std::process::exit(1);
    });
    focus::main(config).await
}
