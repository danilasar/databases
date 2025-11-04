/// Отчётности
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "clarifications")]
pub struct Clarification {
    #[auto]
    #[key]
    id: i32,
    name: String,
}
