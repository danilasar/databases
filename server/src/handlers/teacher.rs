// Обработчики для преподавателей
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::queries::teacher;
use crate::handlers::admin::ApiResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TeacherQuery {
    pub teacher_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CourseQuery {
    pub courses: Vec<i32>,
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
}

// 1. RIGHT JOIN - Предметы с преподавателями
pub async fn get_subjects_with_teachers(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match teacher::get_subjects_with_teachers(&state.db, teacher_id).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка получения предметов с преподавателями: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 2. INTERSECT - Общие студенты у разных преподавателей
pub async fn get_common_students(
    State(state): State<AppState>,
    Path((teacher1_id, teacher2_id)): Path<(i32, i32)>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    if teacher1_id == teacher2_id {
        return Ok(Json(ApiResponse::error("Нельзя сравнивать преподавателя с самим собой".to_string())));
    }
    
    match teacher::get_common_students(&state.db, teacher1_id, teacher2_id).await {
        Ok(students) => Ok(Json(ApiResponse::success(students))),
        Err(e) => {
            eprintln!("Ошибка получения общих студентов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 3. EXCEPT - Студенты без оценок
pub async fn get_students_without_marks(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match teacher::get_students_without_marks(&state.db, teacher_id).await {
        Ok(students) => Ok(Json(ApiResponse::success(students))),
        Err(e) => {
            eprintln!("Ошибка получения студентов без оценок: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 4. IN и BETWEEN - Фильтрация по критериям
pub async fn get_students_by_criteria(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
    Json(query): Json<CourseQuery>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let min_age = query.min_age.unwrap_or(18);
    let max_age = query.max_age.unwrap_or(65);
    
    if query.courses.is_empty() {
        return Ok(Json(ApiResponse::error("Необходимо указать хотя бы один курс".to_string())));
    }
    
    match teacher::get_students_by_criteria(&state.db, teacher_id, query.courses, min_age, max_age).await {
        Ok(students) => Ok(Json(ApiResponse::success(students))),
        Err(e) => {
            eprintln!("Ошибка фильтрации студентов по критериям: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 5. LIKE и ILIKE - Поиск по паттернам
pub async fn search_subjects_and_students(
    State(state): State<AppState>,
    Path(search_pattern): Path<String>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    if search_pattern.trim().is_empty() {
        return Ok(Json(ApiResponse::error("Поисковый запрос не может быть пустым".to_string())));
    }
    
    match teacher::search_subjects_and_students(&state.db, &search_pattern).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            eprintln!("Ошибка поиска предметов и студентов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 6. Самосоединение - Иерархия подразделений
pub async fn get_department_hierarchy(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match teacher::get_department_hierarchy(&state.db).await {
        Ok(hierarchy) => Ok(Json(ApiResponse::success(hierarchy))),
        Err(e) => {
            eprintln!("Ошибка получения иерархии подразделений: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 7. Функции преобразования и NULL - Информация о преподавателе
pub async fn get_formatted_teacher_info(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<ApiResponse<Option<serde_json::Value>>>, StatusCode> {
    match teacher::get_formatted_teacher_info(&state.db, teacher_id).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => {
            eprintln!("Ошибка получения информации о преподавателе: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}