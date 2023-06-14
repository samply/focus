use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = focus::init_logger() {
        eprintln!("Cannot initalize logger: {}", e);
        return ExitCode::from(1);
    };
    match focus::Config::load() {
        Ok(cfg) => focus::main(cfg).await,
        Err(e) => {
            eprintln!("Unable to start as there was an error reading the config:\n{}\n\nTerminating -- please double-check your startup parameters with --help and refer to the documentation.", e);
            ExitCode::from(1)
        }
    }
}
