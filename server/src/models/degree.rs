/// Формы обучения
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "degrees")]
pub struct Degree {
    #[auto]
    #[key]
    id: i32,
    name: Option<String>,
}
