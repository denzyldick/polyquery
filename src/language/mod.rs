use std::collections::HashMap;
use std::sync::Mutex;
use tree_sitter::{Language, Parser, Query};

/// Configuration for a single programming language, including tree-sitter grammar and queries.
pub struct LanguageConfig {
    /// The human-readable name of the language.
    pub name: &'static str,
    /// The file extensions associated with this language.
    pub extensions: &'static [&'static str],
    /// The tree-sitter Language for parsing.
    pub language: Language,
    /// A thread-safe parser instance for this language.
    pub parser: Mutex<Parser>,
    /// An optional tree-sitter query for detecting tagged template literals (e.g., `sql`).
    pub tagged_template_query: Option<(String, Query)>,
    /// An optional tree-sitter query for detecting SQL in comments.
    pub comment_query: Option<Query>,
}

/// Internal constructors for LanguageConfig instances.
impl LanguageConfig {
    fn new(
        name: &'static str,
        extensions: &'static [&'static str],
        language_fn: impl Into<Language>,
        tagged_template_pattern: Option<&str>,
        comment_pattern: Option<&str>,
    ) -> Self {
        let language: Language = language_fn.into();
        let mut parser = Parser::new();
        if let Err(e) = parser.set_language(&language) {
            eprintln!("Failed to set language {}: {}", name, e);
            return Self::new_error(name);
        };

        let tagged_template_query = tagged_template_pattern.and_then(|s| {
            let q = Query::new(&language, s).ok()?;
            Some((s.to_string(), q))
        });

        let comment_query = comment_pattern.and_then(|s| Query::new(&language, s).ok());

        Self {
            name,
            extensions,
            language,
            parser: Mutex::new(parser),
            tagged_template_query,
            comment_query,
        }
    }

    // Fallback to JavaScript when the requested language grammar cannot be loaded.
    fn new_error(name: &'static str) -> Self {
        let language: Language = tree_sitter_javascript::LANGUAGE.into();
        let parser = Mutex::new(Parser::new());
        Self {
            name,
            extensions: &[],
            language,
            parser,
            tagged_template_query: None,
            comment_query: None,
        }
    }
}

fn js_config() -> LanguageConfig {
    LanguageConfig::new(
        "javascript",
        &["js", "jsx", "mjs", "cjs"],
        tree_sitter_javascript::LANGUAGE,
        Some(
            r#"(call_expression function: (identifier) @tag arguments: (template_string) @content)"#,
        ),
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn ts_config() -> LanguageConfig {
    LanguageConfig::new(
        "typescript",
        &["ts"],
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        Some(
            r#"(call_expression function: (identifier) @tag arguments: (template_string) @content)"#,
        ),
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn tsx_config() -> LanguageConfig {
    LanguageConfig::new(
        "tsx",
        &["tsx"],
        tree_sitter_typescript::LANGUAGE_TSX,
        Some(
            r#"(call_expression function: (identifier) @tag arguments: (template_string) @content)"#,
        ),
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn py_config() -> LanguageConfig {
    LanguageConfig::new(
        "python",
        &["py", "pyi", "pyw"],
        tree_sitter_python::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn go_config() -> LanguageConfig {
    LanguageConfig::new(
        "go",
        &["go"],
        tree_sitter_go::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (interpreted_string_literal) @content"#),
    )
}

fn rb_config() -> LanguageConfig {
    LanguageConfig::new(
        "ruby",
        &["rb"],
        tree_sitter_ruby::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn java_config() -> LanguageConfig {
    LanguageConfig::new(
        "java",
        &["java"],
        tree_sitter_java::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn rs_config() -> LanguageConfig {
    LanguageConfig::new(
        "rust",
        &["rs"],
        tree_sitter_rust::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn php_config() -> LanguageConfig {
    LanguageConfig::new(
        "php",
        &["php"],
        tree_sitter_php::LANGUAGE_PHP,
        None,
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn cs_config() -> LanguageConfig {
    LanguageConfig::new(
        "csharp",
        &["cs"],
        tree_sitter_c_sharp::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn cpp_config() -> LanguageConfig {
    LanguageConfig::new(
        "cpp",
        &["cpp", "cc", "cxx", "hpp"],
        tree_sitter_cpp::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn c_config() -> LanguageConfig {
    LanguageConfig::new(
        "c",
        &["c", "h"],
        tree_sitter_c::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn kt_config() -> LanguageConfig {
    LanguageConfig::new(
        "kotlin",
        &["kt", "kts"],
        tree_sitter_kotlin_sg::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (string_template) @content"#),
    )
}

fn scala_config() -> LanguageConfig {
    LanguageConfig::new(
        "scala",
        &["scala"],
        tree_sitter_scala::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

fn swift_config() -> LanguageConfig {
    LanguageConfig::new(
        "swift",
        &["swift"],
        tree_sitter_swift::LANGUAGE,
        None,
        Some(r#"(line_comment) @comment . (string_literal) @content"#),
    )
}

fn ex_config() -> LanguageConfig {
    LanguageConfig::new(
        "elixir",
        &["ex", "exs"],
        tree_sitter_elixir::LANGUAGE,
        None,
        Some(r#"(comment) @comment . (string) @content"#),
    )
}

/// A registry of supported programming languages, indexed by file extension.
pub struct LanguageRegistry {
    by_extension: HashMap<&'static str, usize>,
    configs: Vec<LanguageConfig>,
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRegistry {
    /// Creates a new registry containing all supported language configurations.
    pub fn new() -> Self {
        let configs: Vec<LanguageConfig> = vec![
            js_config(),
            ts_config(),
            tsx_config(),
            py_config(),
            go_config(),
            rb_config(),
            java_config(),
            rs_config(),
            php_config(),
            cs_config(),
            cpp_config(),
            c_config(),
            kt_config(),
            scala_config(),
            swift_config(),
            ex_config(),
        ];

        let mut by_extension = HashMap::new();
        for (i, config) in configs.iter().enumerate() {
            for ext in config.extensions {
                by_extension.insert(*ext, i);
            }
        }

        Self {
            by_extension,
            configs,
        }
    }

    /// Returns the language configuration for the given file extension, if any.
    pub fn get_by_extension(&self, ext: &str) -> Option<&LanguageConfig> {
        self.by_extension.get(ext).map(|&i| &self.configs[i])
    }

    /// Returns a list of all registered file extensions.
    pub fn all_extensions(&self) -> Vec<&'static str> {
        self.by_extension.keys().copied().collect()
    }
}
