/// Типы структурных подразделений
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "unit_types")]
pub struct UnitType {
    #[auto]
    #[key]
    id: i32,
    name: String,
}
