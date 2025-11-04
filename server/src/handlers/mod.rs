// Модуль обработчиков REST API

pub mod admin;
pub mod teacher;
pub mod student;
pub mod common;

// Переэкспортируем основные обработчики
pub use admin::*;
pub use teacher::*;
pub use student::*;
pub use common::*;