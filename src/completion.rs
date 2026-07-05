use crate::schema::Schema;

/// An item in a completion list, representing a SQL keyword, table name, or column name.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The text displayed and inserted for this completion.
    pub label: String,
    /// Additional detail shown alongside the label.
    pub detail: Option<String>,
    /// Optional custom text to insert instead of the label.
    pub insert_text: Option<String>,
}

/// Returns a list of common SQL keywords as completion items.
pub fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        kw("SELECT"),
        kw("FROM"),
        kw("WHERE"),
        kw("INSERT"),
        kw("INTO"),
        kw("VALUES"),
        kw("UPDATE"),
        kw("SET"),
        kw("DELETE"),
        kw("CREATE"),
        kw("TABLE"),
        kw("ALTER"),
        kw("DROP"),
        kw("INDEX"),
        kw("JOIN"),
        kw("LEFT"),
        kw("RIGHT"),
        kw("INNER"),
        kw("OUTER"),
        kw("ON"),
        kw("AND"),
        kw("OR"),
        kw("NOT"),
        kw("IN"),
        kw("IS"),
        kw("NULL"),
        kw("AS"),
        kw("ORDER"),
        kw("BY"),
        kw("GROUP"),
        kw("HAVING"),
        kw("LIMIT"),
        kw("OFFSET"),
        kw("DISTINCT"),
        kw("COUNT"),
        kw("SUM"),
        kw("AVG"),
        kw("MIN"),
        kw("MAX"),
        kw("BETWEEN"),
        kw("LIKE"),
        kw("EXISTS"),
        kw("UNION"),
        kw("ALL"),
        kw("CASE"),
        kw("WHEN"),
        kw("THEN"),
        kw("ELSE"),
        kw("END"),
        kw("ASC"),
        kw("DESC"),
        kw("PRIMARY"),
        kw("KEY"),
        kw("FOREIGN"),
        kw("REFERENCES"),
        kw("CONSTRAINT"),
        kw("DEFAULT"),
        kw("CHECK"),
        kw("UNIQUE"),
        kw("CASCADE"),
        kw("BEGIN"),
        kw("COMMIT"),
        kw("ROLLBACK"),
        kw("TRANSACTION"),
        kw("TRUE"),
        kw("FALSE"),
        kw("CAST"),
        kw("COALESCE"),
        kw("NULLIF"),
    ]
}

fn kw(label: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some("SQL keyword".to_string()),
        insert_text: None,
    }
}

/// Returns completion items for table names from the given schema.
pub fn table_completions(schema: &Schema) -> Vec<CompletionItem> {
    schema
        .tables
        .iter()
        .map(|t| CompletionItem {
            label: t.name.clone(),
            detail: Some("table".to_string()),
            insert_text: None,
        })
        .collect()
}

/// Returns completion items for columns of a specific table from the schema.
pub fn column_completions(schema: &Schema, table_name: &str) -> Vec<CompletionItem> {
    schema
        .get_table(table_name)
        .map(|t| {
            t.columns
                .iter()
                .map(|c| CompletionItem {
                    label: c.name.clone(),
                    detail: Some(format!("{}.{} ({})", t.name, c.name, c.data_type)),
                    insert_text: None,
                })
                .collect()
        })
        .unwrap_or_default()
}
