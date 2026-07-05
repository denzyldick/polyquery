use std::collections::HashMap;

/// Metadata about a single column in a database table.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// The name of the column.
    pub name: String,
    /// The SQL data type of the column.
    pub data_type: String,
    /// Whether the column allows NULL values.
    pub nullable: bool,
}

/// Metadata about a database table, including its columns.
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// The name of the table.
    pub name: String,
    /// The columns belonging to this table.
    pub columns: Vec<ColumnInfo>,
}

/// A database schema containing table and column metadata.
#[derive(Debug, Clone)]
pub struct Schema {
    /// The tables in the schema.
    pub tables: Vec<TableInfo>,
    by_name: HashMap<String, usize>,
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

impl Schema {
    /// Creates an empty schema.
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    /// Adds a table to the schema.
    pub fn add_table(&mut self, table: TableInfo) {
        self.by_name.insert(table.name.clone(), self.tables.len());
        self.tables.push(table);
    }

    /// Returns a reference to the table with the given name, if it exists.
    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.by_name.get(name).map(|&i| &self.tables[i])
    }

    /// Returns a reference to a specific column in a table, if it exists.
    pub fn get_column(&self, table: &str, column: &str) -> Option<&ColumnInfo> {
        self.get_table(table)?
            .columns
            .iter()
            .find(|c| c.name == column)
    }

    /// Returns a list of all table names in the schema.
    pub fn table_names(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.name.as_str()).collect()
    }
}

/// Introspects a database schema from the given database URL.
///
/// Supports PostgreSQL, SQLite, and MySQL databases.
pub async fn introspect(database_url: &str) -> Result<Schema, String> {
    if database_url.starts_with("postgres") || database_url.starts_with("postgresql") {
        introspect_postgres(database_url).await
    } else if database_url.starts_with("sqlite") {
        introspect_sqlite(database_url).await
    } else if database_url.starts_with("mysql") || database_url.starts_with("mariadb") {
        introspect_mysql(database_url).await
    } else {
        Err(format!("Unsupported database URL scheme: {}", database_url))
    }
}

async fn introspect_postgres(url: &str) -> Result<Schema, String> {
    use sqlx::Row;

    let pool = sqlx::PgPool::connect(url)
        .await
        .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

    let rows = sqlx::query(
        r#"
        SELECT table_name, column_name, data_type, is_nullable
        FROM information_schema.columns
        WHERE table_schema = 'public'
        ORDER BY table_name, ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query schema: {}", e))?;

    let mut schema = Schema::new();
    let mut current_table: Option<String> = None;
    let mut table_columns: Vec<ColumnInfo> = Vec::new();

    for row in rows {
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");

        if current_table.as_deref() != Some(&table_name) {
            if let Some(name) = current_table.take() {
                schema.add_table(TableInfo {
                    name,
                    columns: std::mem::take(&mut table_columns),
                });
            }
            current_table = Some(table_name.clone());
        }

        table_columns.push(ColumnInfo {
            name: column_name,
            data_type,
            nullable: is_nullable == "YES",
        });
    }

    if let Some(name) = current_table.take() {
        schema.add_table(TableInfo {
            name,
            columns: table_columns,
        });
    }

    Ok(schema)
}

async fn introspect_sqlite(url: &str) -> Result<Schema, String> {
    let pool = sqlx::SqlitePool::connect(url)
        .await
        .map_err(|e| format!("Failed to connect to SQLite: {}", e))?;

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to list tables: {}", e))?;

    let mut schema = Schema::new();

    for (table_name,) in tables {
        let rows: Vec<(String, String, bool)> =
            sqlx::query_as("SELECT name, type, \"notnull\" FROM pragma_table_info(?)")
                .bind(&table_name)
                .fetch_all(&pool)
                .await
                .map_err(|e| format!("Failed to get columns for {}: {}", table_name, e))?;

        let columns = rows
            .into_iter()
            .map(|(name, data_type, notnull)| ColumnInfo {
                name,
                data_type,
                nullable: !notnull,
            })
            .collect();

        schema.add_table(TableInfo {
            name: table_name,
            columns,
        });
    }

    Ok(schema)
}

async fn introspect_mysql(url: &str) -> Result<Schema, String> {
    use sqlx::Row;

    let pool = sqlx::MySqlPool::connect(url)
        .await
        .map_err(|e| format!("Failed to connect to MySQL: {}", e))?;

    let rows = sqlx::query(
        r#"
        SELECT table_name, column_name, data_type, is_nullable
        FROM information_schema.columns
        WHERE table_schema = DATABASE()
        ORDER BY table_name, ordinal_position
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to query schema: {}", e))?;

    let mut schema = Schema::new();
    let mut current_table: Option<String> = None;
    let mut table_columns: Vec<ColumnInfo> = Vec::new();

    for row in rows {
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");
        let data_type: String = row.get("data_type");
        let is_nullable: String = row.get("is_nullable");

        if current_table.as_deref() != Some(&table_name) {
            if let Some(name) = current_table.take() {
                schema.add_table(TableInfo {
                    name,
                    columns: std::mem::take(&mut table_columns),
                });
            }
            current_table = Some(table_name.clone());
        }

        table_columns.push(ColumnInfo {
            name: column_name,
            data_type,
            nullable: is_nullable == "YES",
        });
    }

    if let Some(name) = current_table.take() {
        schema.add_table(TableInfo {
            name,
            columns: table_columns,
        });
    }

    Ok(schema)
}
