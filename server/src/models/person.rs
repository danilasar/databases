/// Люди
#[derive(Debug, Clone, db_set_macros::DbSet)]
#[dbset(table_name = "people")]
pub struct Person {
    #[auto]
    #[key]
    id: i32,
    surname: String,
    name: String,
    patronymic: Option<String>,
    birthday: Option<chrono::NaiveDate>,
    #[unique]
    snils: Option<i64>,
    #[unique]
    inn: Option<i64>,
}
