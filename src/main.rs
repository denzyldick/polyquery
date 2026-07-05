#![allow(dead_code)]

mod backend;
mod completion;
mod config;
mod execution;
mod language;
mod schema;
mod sql;

use tower_lsp::{LspService, Server};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "polyquery=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting Polyquery LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
