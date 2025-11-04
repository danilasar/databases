/// Направления и специальности
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "specialities")]
pub struct Speciality {
    #[auto]
    #[key]
    id: i32,
    code: String,
    name: String,
    description: Option<String>,
    degree: i32,
}
