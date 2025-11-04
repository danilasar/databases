/// Университеты
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "universities")]
pub struct University {
    #[auto]
    #[key]
    id: i32,
    name: String,
    full_name: String,
    rector: Option<i32>,
}
