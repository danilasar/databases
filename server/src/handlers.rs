// HTTP обработчики для REST API
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::queries;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AgeRangeQuery {
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeacherSubjectQuery {
    pub teacher_id: i32,
    pub subject_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CurrentTermQuery {
    pub current_term: i32,
}

// ========================
// Административные обработчики
// ========================

/// Получить структуру университета
pub async fn get_university_structure(
    State(state): State<AppState>,
) -> Result<Json<Vec<queries::UniversityStructure>>, StatusCode> {
    match queries::admin::get_university_structure(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить всех людей с ролями
pub async fn get_all_people_with_roles(
    State(state): State<AppState>,
) -> Result<Json<Vec<queries::PersonWithRole>>, StatusCode> {
    match queries::admin::get_all_people_with_roles(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить объединение преподавателей и студентов
pub async fn get_teachers_and_students_union(
    State(state): State<AppState>,
) -> Result<Json<Vec<queries::PersonWithRole>>, StatusCode> {
    match queries::admin::get_teachers_and_students_union(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить людей, не связанных с университетом
pub async fn get_people_not_in_university(
    State(state): State<AppState>,
) -> Result<Json<Vec<queries::PersonWithRole>>, StatusCode> {
    match queries::admin::get_people_not_in_university(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Проверить активные дисциплины
pub async fn check_active_subjects(
    State(state): State<AppState>,
) -> Result<Json<Vec<(i32, String, Option<String>)>>, StatusCode> {
    match queries::admin::check_active_subjects(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить людей по возрастному диапазону
pub async fn get_people_by_age_range(
    State(state): State<AppState>,
    Query(params): Query<AgeRangeQuery>,
) -> Result<Json<Vec<queries::PersonAge>>, StatusCode> {
    let min_age = params.min_age.unwrap_or(0);
    let max_age = params.max_age.unwrap_or(120);
    
    match queries::admin::get_people_by_age_range(&state.db, min_age, max_age).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Поиск людей по имени
pub async fn search_people_by_name(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<queries::PersonSearch>>, StatusCode> {
    let search_pattern = params.q.unwrap_or_default();
    
    if search_pattern.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match queries::admin::search_people_by_name(&state.db, &search_pattern).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить статистику университетов
pub async fn get_university_statistics(
    State(state): State<AppState>,
) -> Result<Json<Vec<queries::UniversityStatistics>>, StatusCode> {
    match queries::admin::get_university_statistics(&state.db).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ========================
// Обработчики для преподавателей
// ========================

/// Получить учебную нагрузку преподавателя
pub async fn get_my_teaching_load(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<Vec<queries::TeachingLoad>>, StatusCode> {
    match queries::teacher::get_my_teaching_load(&state.db, teacher_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить студентов преподавателя
pub async fn get_my_students(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<Vec<queries::TeacherStudent>>, StatusCode> {
    match queries::teacher::get_my_students(&state.db, teacher_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить оценки студентов по предмету
pub async fn get_student_marks_by_subject(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<queries::StudentMark>>, StatusCode> {
    let subject_id: i32 = params
        .get("subject_id")
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    match queries::teacher::get_student_marks_by_subject(&state.db, teacher_id, subject_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить предметы, которые преподает преподаватель
pub async fn get_subjects_i_teach(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<Vec<queries::TeacherSubject>>, StatusCode> {
    match queries::teacher::get_subjects_i_teach(&state.db, teacher_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить статистику оценок преподавателя
pub async fn get_teacher_marks_statistics(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<Vec<(String, i64, i64)>>, StatusCode> {
    match queries::teacher::get_teacher_marks_statistics(&state.db, teacher_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить график преподавателя
pub async fn get_teacher_schedule(
    State(state): State<AppState>,
    Path(teacher_id): Path<i32>,
) -> Result<Json<Vec<(i32, String, String, i32)>>, StatusCode> {
    match queries::teacher::get_teacher_schedule(&state.db, teacher_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ========================
// Обработчики для студентов
// ========================

/// Получить учебный план студента
pub async fn get_my_curriculum(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Vec<queries::StudentCurriculum>>, StatusCode> {
    match queries::student::get_my_curriculum(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить одногруппников
pub async fn get_my_group_mates(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Vec<queries::GroupMate>>, StatusCode> {
    match queries::student::get_my_group_mates(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить оценки студента
pub async fn get_my_marks(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Vec<queries::StudentMark>>, StatusCode> {
    match queries::student::get_my_marks(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить предметы текущего семестра
pub async fn get_current_semester_subjects(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
    Query(params): Query<CurrentTermQuery>,
) -> Result<Json<Vec<queries::CurrentSemesterSubject>>, StatusCode> {
    match queries::student::get_current_semester_subjects(&state.db, student_id, params.current_term).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить информацию о специальности
pub async fn get_speciality_info(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Option<queries::SpecialityInfo>>, StatusCode> {
    match queries::student::get_speciality_info(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить прогресс обучения
pub async fn get_study_progress(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Vec<(i32, i64, i64, f64)>>, StatusCode> {
    match queries::student::get_study_progress(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить преподавателей студента
pub async fn get_my_teachers(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
) -> Result<Json<Vec<(String, String, String)>>, StatusCode> {
    match queries::student::get_my_teachers(&state.db, student_id).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Получить долги студента
pub async fn get_my_debts(
    State(state): State<AppState>,
    Path(student_id): Path<i32>,
    Query(params): Query<CurrentTermQuery>,
) -> Result<Json<Vec<(String, i32, String, String)>>, StatusCode> {
    match queries::student::get_my_debts(&state.db, student_id, params.current_term).await {
        Ok(result) => Ok(Json(result)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
