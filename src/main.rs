use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{info, warn};
use tree_sitter::{Parser, Query, QueryCursor};

struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
    parser: Mutex<Parser>,
    language: tree_sitter::Language,
}

impl Backend {
    fn new(client: Client) -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::language_typescript();
        parser
            .set_language(&language)
            .expect("Failed to load TypeScript grammar");
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
            parser: Mutex::new(parser),
            language,
        }
    }

    fn process_document(&self, uri: &Url, source: &str) {
        if !uri.path().ends_with(".ts") && !uri.path().ends_with(".tsx") {
            return;
        }

        let mut parser = self.parser.lock().unwrap();
        let tree = match parser.parse(source, None) {
            Some(t) => t,
            None => {
                warn!("Failed to parse {}", uri);
                return;
            }
        };

        let root = tree.root_node();
        let source_bytes = source.as_bytes();

        let q_tagged = r#"(
          (call_expression
            function: (identifier) @tag
            arguments: (_) @content)
        )"#;

        if let Ok(query) = Query::new(&self.language, q_tagged) {
            let tag_idx = query.capture_index_for_name("tag");
            let content_idx = query.capture_index_for_name("content");
            if let (Some(tag_idx), Some(content_idx)) = (tag_idx, content_idx) {
                let mut cursor = QueryCursor::new();
                for m in cursor.matches(&query, root, source_bytes) {
                    let tag = m.captures.iter().find(|c| c.index == tag_idx);
                    let content = m.captures.iter().find(|c| c.index == content_idx);
                    if let (Some(tag), Some(content)) = (tag, content) {
                        if tag.node.utf8_text(source_bytes).unwrap_or("") != "sql" {
                            continue;
                        }
                        let sql_text = content.node.utf8_text(source_bytes).unwrap_or("");
                        let start = content.node.start_position();
                        let end = content.node.end_position();
                        info!(
                            "SQL tagged template {}:{} - {}:{} => {}",
                            start.row, start.column, end.row, end.column, sql_text
                        );
                    }
                }
            }
        }

        let q_comment = r#"(
          (comment) @comment
          .
          (string) @content
        )"#;

        if let Ok(query) = Query::new(&self.language, q_comment) {
            let comment_idx = query.capture_index_for_name("comment");
            let content_idx = query.capture_index_for_name("content");
            if let (Some(comment_idx), Some(content_idx)) = (comment_idx, content_idx) {
                let mut cursor = QueryCursor::new();
                for m in cursor.matches(&query, root, source_bytes) {
                    let comment = m.captures.iter().find(|c| c.index == comment_idx);
                    let content = m.captures.iter().find(|c| c.index == content_idx);
                    if let (Some(comment), Some(content)) = (comment, content) {
                        let comment_text = comment.node.utf8_text(source_bytes).unwrap_or("");
                        if !comment_text.contains("sql") {
                            continue;
                        }
                        let sql_text = content.node.utf8_text(source_bytes).unwrap_or("");
                        let start = content.node.start_position();
                        let end = content.node.end_position();
                        info!(
                            "SQL comment-annotated {}:{} - {}:{} => {}",
                            start.row, start.column, end.row, end.column, sql_text
                        );
                    }
                }
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Polyquery LSP initializing");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "polyquery".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Polyquery LSP ready");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Polyquery LSP shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        info!("didOpen: {}", uri.path());
        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), params.text_document.text.clone());
        self.process_document(&uri, &params.text_document.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.iter().rfind(|c| c.range.is_none()) {
            info!("didChange (full): {}", uri.path());
            self.documents
                .lock()
                .unwrap()
                .insert(uri.clone(), change.text.clone());
            self.process_document(&uri, &change.text);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("didClose: {}", params.text_document.uri.path());
        self.documents
            .lock()
            .unwrap()
            .remove(&params.text_document.uri);
    }
}

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

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
