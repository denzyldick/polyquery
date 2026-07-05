use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub tables: Vec<TableInfo>,
    by_name: HashMap<String, usize>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    pub fn add_table(&mut self, table: TableInfo) {
        self.by_name
            .insert(table.name.clone(), self.tables.len());
        self.tables.push(table);
    }

    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.by_name.get(name).map(|&i| &self.tables[i])
    }

    pub fn get_column(&self, table: &str, column: &str) -> Option<&ColumnInfo> {
        self.get_table(table)?
            .columns
            .iter()
            .find(|c| c.name == column)
    }

    pub fn table_names(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.name.as_str()).collect()
    }
}
