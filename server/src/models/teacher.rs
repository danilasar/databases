/// Преподаватели
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "teachers")]
pub struct Teacher {
    #[auto]
    #[key]
    id: i32,
    person: i32,
    work: i32,
}
