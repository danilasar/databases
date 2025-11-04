// Обработчики для студентов
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::queries::student;
use crate::handlers::admin::ApiResponse;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SemesterQuery {
    pub semesters: Vec<i32>,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
}

// 1. INNER JOIN - Предметы студента с преподавателями
pub async fn get_student_subjects(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<student::StudentSubject>>>, StatusCode> {
    match student::get_student_subjects(&state.db, student_id).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка получения предметов студента: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 2. LEFT JOIN - Все предметы с информацией о преподавателях
pub async fn get_all_subjects_with_teachers(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_all_subjects_with_teachers(&state.db, student_id).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка получения всех предметов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 3. RIGHT JOIN - Одногруппники с оценками
pub async fn get_classmates_with_marks(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_classmates_with_marks(&state.db, student_id).await {
        Ok(classmates) => Ok(Json(ApiResponse::success(classmates))),
        Err(e) => {
            eprintln!("Ошибка получения одногруппников с оценками: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 4. FULL JOIN - Сравнение успеваемости
pub async fn get_full_group_performance(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_full_group_performance(&state.db, student_id).await {
        Ok(performance) => Ok(Json(ApiResponse::success(performance))),
        Err(e) => {
            eprintln!("Ошибка получения сравнения успеваемости: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 5. CROSS JOIN - Матрица студент-предмет
pub async fn get_student_subject_matrix(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_student_subject_matrix(&state.db, student_id).await {
        Ok(matrix) => Ok(Json(ApiResponse::success(matrix))),
        Err(e) => {
            eprintln!("Ошибка получения матрицы студент-предмет: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 6. Самосоединение - Сравнение с другими студентами
pub async fn compare_with_same_speciality(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::compare_with_same_speciality(&state.db, student_id).await {
        Ok(comparison) => Ok(Json(ApiResponse::success(comparison))),
        Err(e) => {
            eprintln!("Ошибка сравнения с другими студентами: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 7. UNION - Активности студента
pub async fn get_student_activities(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_student_activities(&state.db, student_id).await {
        Ok(activities) => Ok(Json(ApiResponse::success(activities))),
        Err(e) => {
            eprintln!("Ошибка получения активностей студента: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 8. EXCEPT - Несданные предметы
pub async fn get_unsettled_subjects(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_unsettled_subjects(&state.db, student_id).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка получения несданных предметов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 9. INTERSECT - Общие предметы с одногруппниками
pub async fn get_common_subjects_with_classmates(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_common_subjects_with_classmates(&state.db, student_id).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка получения общих предметов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 10. EXISTS - Проверка статуса обучения
pub async fn check_student_performance(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::check_student_performance(&state.db, student_id).await {
        Ok(performance) => Ok(Json(ApiResponse::success(performance))),
        Err(e) => {
            eprintln!("Ошибка проверки статуса обучения: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 11. IN и BETWEEN - Фильтрация по критериям
pub async fn get_subjects_by_criteria(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
    Json(query): Json<SemesterQuery>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let min_length = query.min_length.unwrap_or(1);
    let max_length = query.max_length.unwrap_or(100);
    
    if query.semesters.is_empty() {
        return Ok(Json(ApiResponse::error("Необходимо указать хотя бы один семестр".to_string())));
    }
    
    match student::get_subjects_by_criteria(&state.db, student_id, query.semesters, min_length, max_length).await {
        Ok(subjects) => Ok(Json(ApiResponse::success(subjects))),
        Err(e) => {
            eprintln!("Ошибка фильтрации предметов по критериям: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 12. LIKE и ILIKE - Поиск по паттернам
pub async fn search_subjects_and_classmates(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
    Path(search_pattern): Path<String>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    if search_pattern.trim().is_empty() {
        return Ok(Json(ApiResponse::error("Поисковый запрос не может быть пустым".to_string())));
    }
    
    match student::search_subjects_and_classmates(&state.db, student_id, &search_pattern).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            eprintln!("Ошибка поиска предметов и одногруппников: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 13. Функции даты и времени - Анализ возраста
pub async fn get_student_age_analysis(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Option<serde_json::Value>>>, StatusCode> {
    match student::get_student_age_analysis(&state.db, student_id).await {
        Ok(analysis) => Ok(Json(ApiResponse::success(analysis))),
        Err(e) => {
            eprintln!("Ошибка анализа возраста студента: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 14. Агрегатные функции - Статистика по семестрам
pub async fn get_student_semester_statistics(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match student::get_student_semester_statistics(&state.db, student_id).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            eprintln!("Ошибка получения статистики по семестрам: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 15. Строковые функции - Форматирование информации
pub async fn get_formatted_student_info(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Option<serde_json::Value>>>, StatusCode> {
    match student::get_formatted_student_info(&state.db, student_id).await {
        Ok(info) => Ok(Json(ApiResponse::success(info))),
        Err(e) => {
            eprintln!("Ошибка форматирования информации о студенте: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 16. COALESCE, NULLIF, GREATEST, LEAST - Работа с NULL
pub async fn get_student_data_with_nulls(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Option<serde_json::Value>>>, StatusCode> {
    match student::get_student_data_with_nulls(&state.db, student_id).await {
        Ok(data) => Ok(Json(ApiResponse::success(data))),
        Err(e) => {
            eprintln!("Ошибка обработки данных с NULL: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// === Оригинальные обработчики (совместимость) ===

// Учебный план студента
pub async fn get_my_curriculum(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<student::StudentCurriculum>>>, StatusCode> {
    match student::get_my_curriculum(&state.db, student_id).await {
        Ok(curriculum) => Ok(Json(ApiResponse::success(curriculum))),
        Err(e) => {
            eprintln!("Ошибка получения учебного плана: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Одногруппники
pub async fn get_my_group_mates(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<student::GroupMate>>>, StatusCode> {
    match student::get_my_group_mates(&state.db, student_id).await {
        Ok(mates) => Ok(Json(ApiResponse::success(mates))),
        Err(e) => {
            eprintln!("Ошибка получения одногруппников: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Оценки студента
pub async fn get_my_marks(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<student::StudentMark>>>, StatusCode> {
    match student::get_my_marks(&state.db, student_id).await {
        Ok(marks) => Ok(Json(ApiResponse::success(marks))),
        Err(e) => {
            eprintln!("Ошибка получения оценок: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}