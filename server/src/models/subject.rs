/// Дисциплины
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "subjects")]
pub struct Subject {
    #[auto]
    #[key]
    id: i32,
    name: String,
    description: Option<String>,
}
