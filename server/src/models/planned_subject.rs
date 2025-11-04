/// Запланированные дисциплины
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "planned_subjects")]
pub struct PlannedSubject {
    #[auto]
    #[key]
    id: i32,
    curriculum: i32,
    subject: i32,
    term: i32,
    clarification: i32,
}
