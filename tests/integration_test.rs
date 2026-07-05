use std::process::{Command, Stdio};

#[test]
fn test_binary_exists() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("Failed to run polyquery");
    // LSP server doesn't have --help, but should start and wait for stdin
    // Just verify the binary compiles and runs
    assert!(output.status.success() || output.status.code() == Some(1));
}

#[test]
fn test_sql_validation_valid() {
    let errors = polyquery::sql::validate_sql("SELECT * FROM users WHERE id = 1");
    assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
}

#[test]
fn test_sql_validation_invalid() {
    let errors = polyquery::sql::validate_sql("SELECTT * FROM users");
    assert!(!errors.is_empty(), "Expected errors for invalid SQL");
}

#[test]
fn test_sql_validation_empty() {
    let errors = polyquery::sql::validate_sql("");
    assert!(errors.is_empty(), "Expected no errors for empty SQL");
}

#[test]
fn test_language_registry_extensions() {
    let registry = polyquery::language::LanguageRegistry::new();
    let exts = registry.all_extensions();
    assert!(exts.contains(&"js"));
    assert!(exts.contains(&"ts"));
    assert!(exts.contains(&"py"));
    assert!(exts.contains(&"rs"));
    assert!(
        exts.len() >= 20,
        "Expected many extensions, got {}",
        exts.len()
    );
}

#[test]
fn test_language_registry_get() {
    let registry = polyquery::language::LanguageRegistry::new();
    let js = registry.get_by_extension("js");
    assert!(js.is_some(), "Expected JS config");
    assert_eq!(js.unwrap().name, "javascript");

    let ts = registry.get_by_extension("ts");
    assert!(ts.is_some(), "Expected TS config");
    assert_eq!(ts.unwrap().name, "typescript");

    let unknown = registry.get_by_extension("xyz");
    assert!(
        unknown.is_none(),
        "Expected no config for unknown extension"
    );
}

#[test]
fn test_keyword_completions() {
    let items = polyquery::completion::keyword_completions();
    assert!(!items.is_empty(), "Expected keyword completions");
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"SELECT"));
    assert!(labels.contains(&"FROM"));
    assert!(labels.contains(&"WHERE"));
}

#[test]
fn test_schema_empty() {
    let schema = polyquery::schema::Schema::new();
    assert!(schema.tables.is_empty());
    assert!(schema.table_names().is_empty());
}

#[test]
fn test_schema_add_table() {
    let mut schema = polyquery::schema::Schema::new();
    schema.add_table(polyquery::schema::TableInfo {
        name: "users".to_string(),
        columns: vec![
            polyquery::schema::ColumnInfo {
                name: "id".to_string(),
                data_type: "integer".to_string(),
                nullable: false,
            },
            polyquery::schema::ColumnInfo {
                name: "name".to_string(),
                data_type: "text".to_string(),
                nullable: false,
            },
        ],
    });

    assert_eq!(schema.tables.len(), 1);
    assert_eq!(schema.table_names(), vec!["users"]);

    let users = schema.get_table("users");
    assert!(users.is_some());
    assert_eq!(users.unwrap().columns.len(), 2);

    let col = schema.get_column("users", "name");
    assert!(col.is_some());
    assert_eq!(col.unwrap().data_type, "text");
}

#[test]
fn test_query_result_formatting() {
    let result = polyquery::execution::QueryResult {
        columns: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
        ],
        duration_ms: 5.2,
        row_count: 2,
        error: None,
    };

    let output = polyquery::execution::format_result(&result);
    assert!(output.contains("id | name"));
    assert!(output.contains("Alice"));
    assert!(output.contains("2 rows"));
}

#[test]
fn test_query_result_error() {
    let result = polyquery::execution::QueryResult {
        columns: vec![],
        rows: vec![],
        duration_ms: 0.0,
        row_count: 0,
        error: Some("connection refused".to_string()),
    };

    let output = polyquery::execution::format_result(&result);
    assert!(output.contains("connection refused"));
}

#[ignore]
#[test]
fn test_postgres_schema_introspection() {
    let url = std::env::var("POLYQUERY_DATABASE_URL")
        .expect("POLYQUERY_DATABASE_URL must be set for this test");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let schema = rt.block_on(polyquery::schema::introspect(&url)).unwrap();

    assert!(!schema.tables.is_empty(), "Expected at least one table");
    let names: Vec<&str> = schema.table_names();
    assert!(names.contains(&"users"), "Expected 'users' table");
}

#[ignore]
#[test]
fn test_sqlite_schema_introspection() {
    let url = std::env::var("POLYQUERY_DATABASE_URL")
        .expect("POLYQUERY_DATABASE_URL must be set for this test");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let schema = rt.block_on(polyquery::schema::introspect(&url)).unwrap();

    assert!(!schema.tables.is_empty(), "Expected at least one table");
    let names: Vec<&str> = schema.table_names();
    assert!(names.contains(&"users"), "Expected 'users' table");
}
