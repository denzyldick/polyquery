use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::{info, warn};

use crate::language::LanguageRegistry;
use crate::schema::Schema;
use crate::sql;
use tree_sitter::{QueryCursor, StreamingIterator};

pub struct Backend {
    pub client: Client,
    pub documents: Mutex<HashMap<Url, String>>,
    pub registry: LanguageRegistry,
    pub schema: Mutex<Option<Schema>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let registry = LanguageRegistry::new();
        let ext_count = registry.all_extensions().len();
        info!(
            "Polyquery initialized with {} supported languages",
            ext_count
        );

        Self {
            client,
            documents: Mutex::new(HashMap::new()),
            registry,
            schema: Mutex::new(None),
        }
    }

    async fn load_schema(&self) {
        match std::env::var("POLYQUERY_DATABASE_URL") {
            Ok(url) => {
                info!("Connecting to database for schema introspection...");
                match crate::schema::introspect(&url).await {
                    Ok(s) => {
                        info!("Introspected {} tables from database", s.tables.len());
                        *self.schema.lock().unwrap() = Some(s);
                    }
                    Err(e) => warn!("Failed to introspect schema: {}", e),
                }
            }
            Err(_) => info!("No POLYQUERY_DATABASE_URL set — running without schema awareness"),
        }
    }

    async fn process_document(&self, uri: &Url, source: &str) {
        let path = uri.path();
        let ext = path.rsplit('.').next().unwrap_or("");

        let config = match self.registry.get_by_extension(ext) {
            Some(c) => c,
            None => return,
        };

        let (tree, source_bytes) = {
            let mut parser = config.parser.lock().unwrap();
            match parser.parse(source, None) {
                Some(t) => (t, source.as_bytes()),
                None => {
                    warn!("Failed to parse {}", uri);
                    return;
                }
            }
        };
        let root = tree.root_node();

        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        // Run tagged template query (JS/TS only)
        if let Some((pattern, query)) = &config.tagged_template_query {
            let tag_idx = query.capture_index_for_name("tag");
            let content_idx = query.capture_index_for_name("content");
            if let (Some(tag_idx), Some(content_idx)) = (tag_idx, content_idx) {
                let tag_name = Self::extract_tag_name(pattern);
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(query, root, source_bytes);
                while let Some(m) = matches.next() {
                    let tag = m.captures.iter().find(|c| c.index == tag_idx);
                    let content = m.captures.iter().find(|c| c.index == content_idx);
                    if let (Some(tag), Some(content)) = (tag, content) {
                        if tag.node.utf8_text(source_bytes).unwrap_or("") != tag_name {
                            continue;
                        }
                        let sql_text = content.node.utf8_text(source_bytes).unwrap_or("");
                        let start = content.node.start_position();
                        let end = content.node.end_position();

                        diagnostics.push(self.make_sql_diagnostic(
                            sql_text,
                            start,
                            end,
                            config.name,
                        ));
                    }
                }
            }
        }

        // Run comment-annotated query
        if let Some(query) = &config.comment_query {
            let comment_idx = query.capture_index_for_name("comment");
            let content_idx = query.capture_index_for_name("content");
            if let (Some(comment_idx), Some(content_idx)) = (comment_idx, content_idx) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(query, root, source_bytes);
                while let Some(m) = matches.next() {
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

                        diagnostics.push(self.make_sql_diagnostic(
                            sql_text,
                            start,
                            end,
                            config.name,
                        ));
                    }
                }
            }
        }

        // Publish diagnostics
        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }

    fn extract_tag_name(_pattern: &str) -> &str {
        // Extract the tag name from the pattern (e.g., "sql" from the query)
        "sql"
    }

    fn make_sql_diagnostic(
        &self,
        sql_text: &str,
        start: tree_sitter::Point,
        end: tree_sitter::Point,
        lang: &str,
    ) -> Diagnostic {
        // Validate SQL
        let errors = sql::validate_sql(sql_text);
        let mut sql_range = Range {
            start: Position {
                line: start.row as u32,
                character: start.column as u32,
            },
            end: Position {
                line: end.row as u32,
                character: end.column as u32,
            },
        };

        if let Some(first_err) = errors.first() {
            if let Some(err_range) = &first_err.range {
                sql_range = Range {
                    start: Position {
                        line: (start.row as u32) + err_range.start_point.row as u32,
                        character: err_range.start_point.column as u32,
                    },
                    end: Position {
                        line: (start.row as u32) + err_range.end_point.row as u32,
                        character: err_range.end_point.column as u32,
                    },
                };
            }

            Diagnostic {
                range: sql_range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("polyquery".to_string()),
                message: first_err.message.clone(),
                ..Default::default()
            }
        } else {
            // No SQL errors — show the detected SQL as information
            let preview = if sql_text.len() > 80 {
                format!("{}...", &sql_text[..77])
            } else {
                sql_text.to_string()
            };

            Diagnostic {
                range: sql_range,
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some("polyquery".to_string()),
                message: format!("[{}] SQL: {}", lang, preview),
                ..Default::default()
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
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("polyquery".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        " ".to_string(),
                        ",".to_string(),
                    ]),
                    ..Default::default()
                }),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "polyquery".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.load_schema().await;
        info!("Polyquery LSP ready");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Polyquery LSP shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        info!("didOpen: {}", uri.path());
        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), params.text_document.text.clone());
        self.process_document(&uri, &params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.iter().rfind(|c| c.range.is_none()) {
            info!("didChange: {}", uri.path());
            self.documents
                .lock()
                .unwrap()
                .insert(uri.clone(), change.text.clone());
            self.process_document(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("didClose: {}", params.text_document.uri.path());
        self.documents
            .lock()
            .unwrap()
            .remove(&params.text_document.uri);
    }

    async fn completion(
        &self,
        _: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let mut items: Vec<CompletionItem> = crate::completion::keyword_completions()
            .into_iter()
            .map(|c| CompletionItem {
                label: c.label,
                detail: c.detail,
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            })
            .collect();

        if let Some(schema) = self.schema.lock().unwrap().as_ref() {
            for table in &schema.tables {
                items.push(CompletionItem {
                    label: table.name.clone(),
                    detail: Some("table".to_string()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    kind: Some(CompletionItemKind::CLASS),
                    ..Default::default()
                });
                for col in &table.columns {
                    let detail = format!("{}.{} ({})", table.name, col.name, col.data_type);
                    items.push(CompletionItem {
                        label: col.name.clone(),
                        detail: Some(detail),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        kind: Some(CompletionItemKind::PROPERTY),
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        if params.command == "polyquery.runQuery" {
            let uri = params.arguments.first().and_then(|v| v.as_str());
            let sql = params.arguments.get(1).and_then(|v| v.as_str());

            if let (Some(_uri), Some(sql)) = (uri, sql) {
                let database_url = match std::env::var("POLYQUERY_DATABASE_URL") {
                    Ok(url) => url,
                    Err(_) => {
                        let msg = serde_json::json!({
                            "type": "error",
                            "message": "POLYQUERY_DATABASE_URL not set"
                        });
                        return Ok(Some(msg));
                    }
                };

                let schema = self.schema.lock().unwrap().clone();
                let result = crate::execution::execute_query(&database_url, sql, schema.as_ref()).await;
                let output = crate::execution::format_result(&result);
                let response = serde_json::json!({ "type": "result", "text": output });
                return Ok(Some(response));
            }
        }

        Ok(None)
    }

    async fn code_lens(
        &self,
        params: CodeLensParams,
    ) -> Result<Option<Vec<CodeLens>>> {
        let documents = self.documents.lock().unwrap();
        let source = match documents.get(&params.text_document.uri) {
            Some(s) => s.clone(),
            None => return Ok(None),
        };
        drop(documents);

        let uri = params.text_document.uri;
        let path = uri.path();
        let ext = path.rsplit('.').next().unwrap_or("");
        let config = match self.registry.get_by_extension(ext) {
            Some(c) => c,
            None => return Ok(None),
        };

        let source_bytes = source.as_bytes();
        let mut lenses = Vec::new();

        // Find SQL strings and add "Run Query" code lens
        if let Some((_, query)) = &config.tagged_template_query {
            let content_idx = query.capture_index_for_name("content");
            if let Some(content_idx) = content_idx {
                let mut parser = config.parser.lock().unwrap();
                if let Some(tree) = parser.parse(&source, None) {
                    let root = tree.root_node();
                    let mut cursor = QueryCursor::new();
                    let mut matches = cursor.matches(query, root, source_bytes);
                    while let Some(m) = matches.next() {
                        if let Some(content) = m.captures.iter().find(|c| c.index == content_idx) {
                            let start = content.node.start_position();
                            let sql_text =
                                content.node.utf8_text(source_bytes).unwrap_or("");
                            let trimmed = sql_text.trim();
                            if !trimmed.is_empty() {
                                let range = Range {
                                    start: Position {
                                        line: start.row as u32,
                                        character: start.column as u32,
                                    },
                                    end: Position {
                                        line: start.row as u32,
                                        character: start.column as u32 + 1,
                                    },
                                };
                                lenses.push(CodeLens {
                                    range,
                                    command: Some(Command {
                                        title: "▶ Run Query".to_string(),
                                        command: "polyquery.runQuery".to_string(),
                                        arguments: Some(vec![
                                            serde_json::json!(uri.to_string()),
                                            serde_json::json!(sql_text),
                                        ]),
                                    }),
                                    data: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(Some(lenses))
    }
}
