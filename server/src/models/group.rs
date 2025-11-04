/// Академическая группа
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "groups")]
pub struct Group {
    #[auto]
    #[key]
    id: i32,
    course: i32,
    affilation: i32,
    speciality: i32,
    curriculum: i32,
}
