// Общие обработчики
use axum::{
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};

/// Проверка здоровья API
pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "OK",
        "message": "АПИ университетской базы данных работает",
        "timestamp": chrono::Utc::now()
    })))
}

/// Информация о версии API
pub async fn get_version() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "name": "University Database API",
        "version": "0.1.0",
        "description": "АПИ для работы с базой данных университета",
        "author": "Danila Grigoriev",
        "build_date": env!("VERGEN_BUILD_DATE", "Недоступно"),
        "rust_version": env!("VERGEN_RUSTC_SEMVER", "Недоступно"),
        "supported_roles": [
            "admin",
            "teacher", 
            "student"
        ],
        "features": {
            "joins": ["INNER", "LEFT", "RIGHT", "FULL", "CROSS", "LATERAL", "Самосоединение"],
            "set_operations": ["UNION", "UNION ALL", "EXCEPT", "INTERSECT"],
            "predicates": ["EXISTS", "IN", "BETWEEN", "LIKE", "ILIKE"],
            "functions": {
                "type_conversion": ["CAST", "::"],
                "null_handling": ["COALESCE", "NULLIF", "GREATEST", "LEAST"],
                "string_functions": ["LENGTH", "CHR", "STRPOS", "OVERLAY", "SUBSTRING", "REPLACE", "UPPER", "LOWER", "BTRIM", "LTRIM"],
                "datetime_functions": ["NOW", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP", "AGE", "DATE_PART", "EXTRACT", "LOCALTIMESTAMP"],
                "aggregate_functions": ["MIN", "MAX", "AVG", "SUM", "COUNT", "GROUP BY", "HAVING"]
            }
        }
    })))
}

/// Описание доступных эндпоинтов
pub async fn get_endpoints() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "endpoints": {
            "common": {
                "health": "GET /api/health",
                "version": "GET /api/version",
                "endpoints": "GET /api/endpoints"
            },
            "admin": {
                "university_statistics": "GET /api/admin/university-stats",
                "teaching_load": "GET /api/admin/teaching-load", 
                "structural_hierarchy": "GET /api/admin/structural-hierarchy",
                "people_roles": "GET /api/admin/people-roles",
                "teachers_workload": "GET /api/admin/teachers-workload",
                "search_people": "GET /api/admin/search-people/{term}",
                "age_statistics": "GET /api/admin/age-statistics",
                "speciality_stats": "GET /api/admin/speciality-stats"
            },
            "teacher": {
                "subjects_with_teachers": "GET /api/teacher/{id}/subjects",
                "common_students": "GET /api/teacher/common-students/{id1}/{id2}",
                "students_without_marks": "GET /api/teacher/{id}/students-without-marks",
                "students_by_criteria": "POST /api/teacher/{id}/students-by-criteria",
                "search": "GET /api/teacher/search/{pattern}",
                "department_hierarchy": "GET /api/teacher/department-hierarchy",
                "teacher_info": "GET /api/teacher/{id}/info"
            },
            "student": {
                "subjects": "GET /api/student/{id}/subjects",
                "all_subjects_with_teachers": "GET /api/student/{id}/all-subjects",
                "classmates_with_marks": "GET /api/student/{id}/classmates-marks",
                "group_performance": "GET /api/student/{id}/group-performance",
                "subject_matrix": "GET /api/student/{id}/subject-matrix",
                "compare_speciality": "GET /api/student/{id}/compare-speciality",
                "activities": "GET /api/student/{id}/activities",
                "unsettled_subjects": "GET /api/student/{id}/unsettled-subjects",
                "common_subjects": "GET /api/student/{id}/common-subjects",
                "performance_check": "GET /api/student/{id}/performance-check",
                "subjects_by_criteria": "POST /api/student/{id}/subjects-by-criteria",
                "search": "GET /api/student/{id}/search/{pattern}",
                "age_analysis": "GET /api/student/{id}/age-analysis",
                "semester_stats": "GET /api/student/{id}/semester-stats",
                "formatted_info": "GET /api/student/{id}/formatted-info",
                "data_with_nulls": "GET /api/student/{id}/data-with-nulls",
                "curriculum": "GET /api/student/{id}/curriculum",
                "group_mates": "GET /api/student/{id}/group-mates",
                "marks": "GET /api/student/{id}/marks"
            }
        }
    })))
}