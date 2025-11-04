/// Структурные подразделения университета
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "units")]
pub struct Unit {
    #[auto]
    #[key]
    id: i32,
    name: String,
    university: i32,
    r#type: i32,
    parent: Option<i32>,
    head: Option<i32>,
}
