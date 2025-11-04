/// Студенты
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "students")]
pub struct Student {
    #[auto]
    #[key]
    id: i32,
    person: i32,
    group: i32,
}
