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

    // Header row
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

    // Data rows
    for row in &result.rows {
        let line = row.join(" | ");
        output.push_str(&line);
        output.push('\n');
    }

    output.push_str(&format!("\n{} rows in {:.1}ms", result.row_count, result.duration_ms));
    output
}
