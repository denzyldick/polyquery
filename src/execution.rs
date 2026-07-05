use crate::schema::Schema;
use sqlx::Column;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub duration_ms: f64,
    pub row_count: usize,
    pub error: Option<String>,
}

pub fn format_result(result: &QueryResult) -> String {
    if let Some(err) = &result.error {
        return format!("Error: {}", err);
    }

    let mut output = String::new();

    let header = result
        .columns
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    output.push_str(&header);
    output.push('\n');
    output.push_str(&"-".repeat(header.len()));
    output.push('\n');

    for row in &result.rows {
        let line = row.join(" | ");
        output.push_str(&line);
        output.push('\n');
    }

    output.push_str(&format!(
        "\n{} rows in {:.1}ms",
        result.row_count, result.duration_ms
    ));
    output
}

pub async fn execute_query(database_url: &str, sql: &str, _schema: Option<&Schema>) -> QueryResult {
    let start = Instant::now();

    if sql.trim().to_uppercase().starts_with("SELECT")
        || sql.trim().to_uppercase().starts_with("WITH")
        || sql.trim().to_uppercase().starts_with("EXPLAIN")
    {
        execute_select(database_url, sql, start).await
    } else {
        execute_mutating(database_url, sql, start).await
    }
}

async fn execute_select(database_url: &str, sql: &str, start: Instant) -> QueryResult {
    let result = if database_url.starts_with("postgres") || database_url.starts_with("postgresql") {
        pg_select(database_url, sql).await
    } else if database_url.starts_with("sqlite") {
        sqlite_select(database_url, sql).await
    } else if database_url.starts_with("mysql") || database_url.starts_with("mariadb") {
        mysql_select(database_url, sql).await
    } else {
        return QueryResult {
            columns: vec![],
            rows: vec![],
            duration_ms: 0.0,
            row_count: 0,
            error: Some("Unsupported database".to_string()),
        };
    };

    let (columns, rows) = match result {
        Ok(r) => r,
        Err(e) => {
            return QueryResult {
                columns: vec![],
                rows: vec![],
                duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                row_count: 0,
                error: Some(e),
            }
        }
    };

    let row_count = rows.len();
    QueryResult {
        columns,
        rows,
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        row_count,
        error: None,
    }
}

async fn execute_mutating(database_url: &str, sql: &str, start: Instant) -> QueryResult {
    let error = if database_url.starts_with("postgres") || database_url.starts_with("postgresql") {
        pg_execute(database_url, sql).await
    } else if database_url.starts_with("sqlite") {
        sqlite_execute(database_url, sql).await
    } else if database_url.starts_with("mysql") || database_url.starts_with("mariadb") {
        mysql_execute(database_url, sql).await
    } else {
        Some("Unsupported database".to_string())
    };

    QueryResult {
        columns: vec![],
        rows: vec![],
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
        row_count: 0,
        error,
    }
}

async fn pg_select(url: &str, sql: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use sqlx::Row;
    let pool = sqlx::PgPool::connect(url)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let rows = sqlx::query(sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let columns: Vec<String> = if let Some(row) = rows.first() {
        row.columns().iter().map(|c| c.name().to_string()).collect()
    } else {
        vec![]
    };

    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|col| {
                    let val: Option<String> = row.try_get(col.as_str()).ok();
                    val.unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Ok((columns, data))
}

async fn pg_execute(url: &str, sql: &str) -> Option<String> {
    let pool = sqlx::PgPool::connect(url).await.ok()?;
    sqlx::query(sql)
        .execute(&pool)
        .await
        .err()
        .map(|e| e.to_string())
}

async fn sqlite_select(url: &str, sql: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use sqlx::Row;
    let pool = sqlx::SqlitePool::connect(url)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let rows = sqlx::query(sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let columns: Vec<String> = if let Some(row) = rows.first() {
        row.columns().iter().map(|c| c.name().to_string()).collect()
    } else {
        vec![]
    };

    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|col| {
                    let val: Option<String> = row.try_get(col.as_str()).ok();
                    val.unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Ok((columns, data))
}

async fn sqlite_execute(url: &str, sql: &str) -> Option<String> {
    let pool = sqlx::SqlitePool::connect(url).await.ok()?;
    sqlx::query(sql)
        .execute(&pool)
        .await
        .err()
        .map(|e| e.to_string())
}

async fn mysql_select(url: &str, sql: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    use sqlx::Row;
    let pool = sqlx::MySqlPool::connect(url)
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let rows = sqlx::query(sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Query failed: {}", e))?;

    let columns: Vec<String> = if let Some(row) = rows.first() {
        row.columns().iter().map(|c| c.name().to_string()).collect()
    } else {
        vec![]
    };

    let data: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .map(|col| {
                    let val: Option<String> = row.try_get(col.as_str()).ok();
                    val.unwrap_or_default()
                })
                .collect()
        })
        .collect();

    Ok((columns, data))
}

async fn mysql_execute(url: &str, sql: &str) -> Option<String> {
    let pool = sqlx::MySqlPool::connect(url).await.ok()?;
    sqlx::query(sql)
        .execute(&pool)
        .await
        .err()
        .map(|e| e.to_string())
}
