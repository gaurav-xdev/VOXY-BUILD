use crate::storage::Value;

/// Validates that a SQL identifier contains only safe characters (alphanumeric and underscore).
/// Prevents SQL injection via table/column names.
fn validate_identifier(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Allowed SQL comparison operators to prevent injection via WHERE clauses.
const ALLOWED_OPERATORS: &[&str] = &[
    "=", "!=", "<>", "<", ">", "<=", ">=", "LIKE", "IS", "IS NOT", "IN", "NOT IN", "BETWEEN",
];

fn validate_operator(op: &str) -> bool {
    ALLOWED_OPERATORS.contains(&op)
}

#[derive(Debug, Clone, PartialEq)]
enum QueryOp {
    Insert,
    Select,
}

#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    operation: QueryOp,
    columns: Vec<String>,
    values: Vec<Value>,
    where_clauses: Vec<(String, String, Value)>,
    order_by: Vec<(String, bool)>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    pub fn insert_into(table: impl Into<String>) -> Self {
        let table = table.into();
        assert!(
            validate_identifier(&table),
            "SQL injection attempt rejected: invalid table name '{table}'"
        );
        Self {
            table,
            operation: QueryOp::Insert,
            columns: Vec::new(),
            values: Vec::new(),
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn select_from(table: impl Into<String>) -> Self {
        let table = table.into();
        assert!(validate_identifier(&table), "Invalid table name: {}", table);
        Self {
            table,
            operation: QueryOp::Select,
            columns: Vec::new(),
            values: Vec::new(),
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn column(mut self, col: impl Into<String>) -> Self {
        let col = col.into();
        assert!(validate_identifier(&col), "Invalid column name: {}", col);
        self.columns.push(col);
        self
    }

    pub fn set(mut self, col: impl Into<String>, val: impl Into<Value>) -> Self {
        let col = col.into();
        assert!(validate_identifier(&col), "Invalid column name: {}", col);
        self.columns.push(col);
        self.values.push(val.into());
        self
    }

    pub fn r#where(
        mut self,
        col: impl Into<String>,
        op: impl Into<String>,
        val: impl Into<Value>,
    ) -> Self {
        let col = col.into();
        let op = op.into();
        assert!(
            validate_identifier(&col),
            "Invalid column name in WHERE: {}",
            col
        );
        assert!(validate_operator(&op), "Invalid operator in WHERE: {}", op);
        self.where_clauses.push((col, op, val.into()));
        self
    }

    pub fn order_by(mut self, col: impl Into<String>, ascending: bool) -> Self {
        let col = col.into();
        assert!(
            validate_identifier(&col),
            "Invalid column name in ORDER BY: {}",
            col
        );
        self.order_by.push((col, ascending));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    pub fn build(&self) -> (String, Vec<Value>) {
        match self.operation {
            QueryOp::Insert => self.build_insert(),
            QueryOp::Select => self.build_select(),
        }
    }

    fn build_insert(&self) -> (String, Vec<Value>) {
        let placeholders: Vec<String> = (0..self.values.len()).map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(", ");
        if self.columns.is_empty() {
            let sql = format!("INSERT INTO {} VALUES ({})", self.table, placeholders_str);
            (sql, self.values.clone())
        } else {
            let cols_str = self.columns.join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.table, cols_str, placeholders_str
            );
            (sql, self.values.clone())
        }
    }

    fn build_select(&self) -> (String, Vec<Value>) {
        let cols_str = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", cols_str, self.table);
        let mut params = Vec::new();

        if !self.where_clauses.is_empty() {
            let conditions: Vec<String> = self
                .where_clauses
                .iter()
                .map(|(c, op, _)| format!("{} {} ?", c, op))
                .collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
            for (_, _, v) in &self.where_clauses {
                params.push(v.clone());
            }
        }

        if !self.order_by.is_empty() {
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|(c, asc)| format!("{} {}", c, if *asc { "ASC" } else { "DESC" }))
                .collect();
            sql.push_str(&format!(" ORDER BY {}", orders.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    pub fn table(&self) -> &str {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_insert() {
        let (sql, params) = QueryBuilder::insert_into("users")
            .set("name", "Alice")
            .set("age", 30i64)
            .build();

        assert_eq!(sql, "INSERT INTO users (name, age) VALUES (?, ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_select_all() {
        let (sql, params) = QueryBuilder::select_from("users").build();

        assert_eq!(sql, "SELECT * FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_select_columns() {
        let (sql, params) = QueryBuilder::select_from("users")
            .column("name")
            .column("age")
            .build();

        assert_eq!(sql, "SELECT name, age FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_where() {
        let (sql, params) = QueryBuilder::select_from("users")
            .r#where("age", ">", 18i64)
            .build();

        assert_eq!(sql, "SELECT * FROM users WHERE age > ?");
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0], Value::I64(18)));
    }

    #[test]
    fn test_build_where_multiple() {
        let (sql, params) = QueryBuilder::select_from("users")
            .r#where("age", ">=", 18i64)
            .r#where("status", "=", "active")
            .build();

        assert_eq!(sql, "SELECT * FROM users WHERE age >= ? AND status = ?");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_build_order_by() {
        let (sql, params) = QueryBuilder::select_from("users")
            .order_by("name", true)
            .build();

        assert_eq!(sql, "SELECT * FROM users ORDER BY name ASC");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_order_by_desc() {
        let (sql, _) = QueryBuilder::select_from("users")
            .order_by("age", false)
            .build();
        assert_eq!(sql, "SELECT * FROM users ORDER BY age DESC");
    }

    #[test]
    fn test_build_limit_offset() {
        let (sql, params) = QueryBuilder::select_from("posts")
            .limit(10)
            .offset(20)
            .build();

        assert_eq!(sql, "SELECT * FROM posts LIMIT 10 OFFSET 20");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_compound() {
        let (sql, params) = QueryBuilder::select_from("posts")
            .column("id")
            .column("title")
            .r#where("published", "=", true)
            .order_by("created_at", false)
            .limit(5)
            .build();

        assert_eq!(
            sql,
            "SELECT id, title FROM posts WHERE published = ? ORDER BY created_at DESC LIMIT 5"
        );
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_build_insert_without_columns() {
        let (sql, params) = QueryBuilder::insert_into("logs")
            .set("message", "hello")
            .build();

        assert_eq!(sql, "INSERT INTO logs (message) VALUES (?)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_parameterized_query() {
        let (sql, params) = QueryBuilder::select_from("items")
            .r#where("category", "=", "books")
            .r#where("price", "<=", 50f64)
            .build();

        assert!(sql.contains("?"));
        assert_eq!(params.len(), 2);
        match &params[0] {
            Value::String(s) => assert_eq!(s, "books"),
            _ => panic!("Expected string param"),
        }
        match &params[1] {
            Value::F64(f) => assert!((*f - 50.0).abs() < 0.001),
            _ => panic!("Expected f64 param"),
        }
    }

    #[test]
    fn test_table_name() {
        let qb = QueryBuilder::select_from("my_table");
        assert_eq!(qb.table(), "my_table");
    }
}
