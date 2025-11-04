// Модуль с всеми запросами к базе данных

pub mod admin;
pub mod teacher;
pub mod student;

// Переэкспортируем основные типы
pub use admin::*;
pub use teacher::*;
pub use student::*;
