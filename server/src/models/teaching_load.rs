/// Учебная нагрузка
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "teaching_load")]
pub struct TeachingLoad {
    planned_subject: i32,
    teacher: i32,
}
