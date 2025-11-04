/// Оценки
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "marks")]
pub struct Mark {
    #[auto]
    #[key]
    id: i32,
    student: i32,
    subject: i32,
    teacher: i32,
}
