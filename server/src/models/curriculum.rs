/// Учебные планы
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "curriculums")]
pub struct Curriculum {
    #[auto]
    #[key]
    id: i32,
    speciality: Option<i32>,
}
