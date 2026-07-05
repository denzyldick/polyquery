use crate::schema::Schema;

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

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
