use std::sync::Mutex;
use tree_sitter::{Parser, Range};

static SQL_LANGUAGE: std::sync::LazyLock<tree_sitter::Language> =
    std::sync::LazyLock::new(|| tree_sitter_sequel::LANGUAGE.into());

static SQL_PARSER: std::sync::LazyLock<Mutex<Parser>> = std::sync::LazyLock::new(|| {
    let mut parser = Parser::new();
    parser.set_language(&SQL_LANGUAGE).ok();
    Mutex::new(parser)
});

#[derive(Debug)]
pub struct SqlError {
    pub message: String,
    pub range: Option<Range>,
}

pub fn validate_sql(sql_text: &str) -> Vec<SqlError> {
    let mut parser = SQL_PARSER.lock().unwrap();
    let tree = match parser.parse(sql_text, None) {
        Some(t) => t,
        None => return vec![],
    };

    let mut errors = Vec::new();
    let root = tree.root_node();

    if root.has_error() {
        // Walk the tree to find ERROR nodes
        find_errors(root, sql_text, &mut errors);
    }

    errors
}

fn find_errors(node: tree_sitter::Node, source: &str, errors: &mut Vec<SqlError>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "ERROR" || child.is_missing() {
            let start = child.start_position();
            let end = child.end_position();
            let context = child.utf8_text(source.as_bytes()).unwrap_or("");
            errors.push(SqlError {
                message: format!("Unexpected SQL syntax near: {}", context),
                range: Some(Range {
                    start_byte: child.start_byte(),
                    end_byte: child.end_byte(),
                    start_point: start,
                    end_point: end,
                }),
            });
        }
        if child.child_count() > 0 {
            find_errors(child, source, errors);
        }
    }
}

pub fn get_sql_parser() -> std::sync::MutexGuard<'static, Parser> {
    SQL_PARSER.lock().unwrap()
}
