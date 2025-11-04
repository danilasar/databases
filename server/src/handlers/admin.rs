// Обработчики для администратора
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::queries::admin;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct AgeQuery {
    pub min_age: Option<i32>,
    pub max_age: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }
    
    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

// 1. INNER JOIN - Статистика по университетам
pub async fn get_university_statistics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<admin::UniversityStats>>>, StatusCode> {
    match admin::get_university_statistics(&state.db).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            eprintln!("Ошибка получения статистики университетов: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 2. FULL JOIN - Нагрузка преподавателей
pub async fn get_full_teaching_load(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<admin::TeachingLoad>>>, StatusCode> {
    match admin::get_full_teaching_load(&state.db).await {
        Ok(load) => Ok(Json(ApiResponse::success(load))),
        Err(e) => {
            eprintln!("Ошибка получения нагрузки преподавателей: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 3. CROSS JOIN LATERAL - Структурная иерархия
pub async fn get_structural_hierarchy(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<admin::StructuralUnit>>>, StatusCode> {
    match admin::get_structural_hierarchy(&state.db).await {
        Ok(hierarchy) => Ok(Json(ApiResponse::success(hierarchy))),
        Err(e) => {
            eprintln!("Ошибка получения структурной иерархии: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 4. UNION ALL - Все люди с ролями
pub async fn get_all_people_with_roles(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match admin::get_all_people_with_roles(&state.db).await {
        Ok(people) => Ok(Json(ApiResponse::success(people))),
        Err(e) => {
            eprintln!("Ошибка получения списка людей с ролями: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 5. EXISTS и CASE - Статус нагрузки преподавателей
pub async fn get_teachers_workload_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match admin::get_teachers_workload_status(&state.db).await {
        Ok(status) => Ok(Json(ApiResponse::success(status))),
        Err(e) => {
            eprintln!("Ошибка получения статуса нагрузки: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 6. Строковые функции - Поиск людей
pub async fn search_people_formatted(
    State(state): State<AppState>,
    Path(term): Path<String>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    if term.trim().is_empty() {
        return Ok(Json(ApiResponse::error("Поисковый запрос не может быть пустым".to_string())));
    }
    
    match admin::search_people_formatted(&state.db, &term).await {
        Ok(people) => Ok(Json(ApiResponse::success(people))),
        Err(e) => {
            eprintln!("Ошибка поиска людей: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 7. Функции даты и времени - Статистика по возрасту
pub async fn get_age_statistics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match admin::get_age_statistics(&state.db).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            eprintln!("Ошибка получения статистики по возрасту: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 8. Агрегатные функции - Статистика по специальностям
pub async fn get_speciality_statistics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    match admin::get_speciality_statistics(&state.db).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            eprintln!("Ошибка получения статистики по специальностям: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}