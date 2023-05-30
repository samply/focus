
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    focus::main().await
}
