// Основной файл сервера университетской базы данных
use axum::{extract::Extension, http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method}, routing::{get, post}, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tokio::net::TcpListener;

mod queries;
mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализируем логирование
    env_logger::init();
    
    println!("🚀 Запуск сервера университетской базы данных...");
    
    // Получаем URL базы данных
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/university".to_string()
        });
    
    println!("📊 Подключение к базе данных...");
    
    // Создаем пул соединений с PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("❌ Не удалось подключиться к PostgreSQL");
    
    println!("✅ Успешное подключение к базе данных");
    
    let app_state = AppState { db: pool };
    
    // Настраиваем CORS
    let cors = CorsLayer::new()
        .allow_origin(
            env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .parse::<HeaderValue>()
                .unwrap()
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
    
    // Создаем основной роутер с полным набором эндпоинтов
    let app = Router::new()
        // === Общие маршруты ===
        .route("/api/health", get(handlers::common::health_check))
        .route("/api/version", get(handlers::common::get_version))
        .route("/api/endpoints", get(handlers::common::get_endpoints))
        
        // === Административные маршруты ===
        .route("/api/admin/university-statistics", get(handlers::admin::get_university_statistics))
        .route("/api/admin/teaching-load", get(handlers::admin::get_full_teaching_load))
        .route("/api/admin/structural-hierarchy", get(handlers::admin::get_structural_hierarchy))
        .route("/api/admin/people-roles", get(handlers::admin::get_all_people_with_roles))
        .route("/api/admin/teachers-workload", get(handlers::admin::get_teachers_workload_status))
        .route("/api/admin/search-people/:term", get(handlers::admin::search_people_formatted))
        .route("/api/admin/age-statistics", get(handlers::admin::get_age_statistics))
        .route("/api/admin/speciality-stats", get(handlers::admin::get_speciality_statistics))
        
        // === Маршруты для преподавателей ===
        .route("/api/teacher/:id/subjects", get(handlers::teacher::get_subjects_with_teachers))
        .route("/api/teacher/common-students/:id1/:id2", get(handlers::teacher::get_common_students))
        .route("/api/teacher/:id/students-without-marks", get(handlers::teacher::get_students_without_marks))
        .route("/api/teacher/:id/students-by-criteria", post(handlers::teacher::get_students_by_criteria))
        .route("/api/teacher/search/:pattern", get(handlers::teacher::search_subjects_and_students))
        .route("/api/teacher/department-hierarchy", get(handlers::teacher::get_department_hierarchy))
        .route("/api/teacher/:id/info", get(handlers::teacher::get_formatted_teacher_info))
        
        // === Маршруты для студентов ===
        // Новые расширенные маршруты
        .route("/api/student/:id/subjects-all", get(handlers::student::get_all_subjects_with_teachers))
        .route("/api/student/:id/classmates-with-marks", get(handlers::student::get_classmates_with_marks))
        .route("/api/student/:id/group-performance", get(handlers::student::get_full_group_performance))
        .route("/api/student/:id/subject-matrix", get(handlers::student::get_student_subject_matrix))
        .route("/api/student/:id/compare-speciality", get(handlers::student::compare_with_same_speciality))
        .route("/api/student/:id/activities", get(handlers::student::get_student_activities))
        .route("/api/student/:id/unsettled-subjects", get(handlers::student::get_unsettled_subjects))
        .route("/api/student/:id/common-subjects", get(handlers::student::get_common_subjects_with_classmates))
        .route("/api/student/:id/performance-check", get(handlers::student::check_student_performance))
        .route("/api/student/:id/subjects-by-criteria", post(handlers::student::get_subjects_by_criteria))
        .route("/api/student/:id/search/:pattern", get(handlers::student::search_subjects_and_classmates))
        .route("/api/student/:id/age-analysis", get(handlers::student::get_student_age_analysis))
        .route("/api/student/:id/semester-stats", get(handlers::student::get_student_semester_statistics))
        .route("/api/student/:id/formatted-info", get(handlers::student::get_formatted_student_info))
        .route("/api/student/:id/data-with-nulls", get(handlers::student::get_student_data_with_nulls))
        
        // Оригинальные маршруты для совместимости
        .route("/api/student/:id/curriculum", get(handlers::student::get_my_curriculum))
        .route("/api/student/:id/group-mates", get(handlers::student::get_my_group_mates))
        .route("/api/student/:id/marks", get(handlers::student::get_my_marks))
        .route("/api/student/:id/current-semester/:term", get(handlers::student::get_current_semester_subjects))
        .route("/api/student/:id/speciality-info", get(handlers::student::get_speciality_info))
        .route("/api/student/:id/study-progress", get(handlers::student::get_study_progress))
        .route("/api/student/:id/teachers", get(handlers::student::get_my_teachers))
        .route("/api/student/:id/debts/:term", get(handlers::student::get_my_debts))
        .route("/api/student/:id/subjects", get(handlers::student::get_student_subjects))
        .route("/api/student/:id/group-info", get(handlers::student::get_student_group_info))
        .route("/api/student/:id/marks-history", get(handlers::student::get_student_marks_history))
        
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(Extension(app_state))
        );
    
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse::<u16>()
        .expect("PORT должен быть числом");
    
    let addr = format!("0.0.0.0:{}", port);
    println!("🌐 Сервер запускается на http://{}", addr);
    println!("📖 Документация API доступна на http://{}/api/endpoints", addr);
    
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Не удалось привязаться к адресу");
    
    println!("✅ Сервер успешно запущен! Готов к обработке запросов.");
    println!("🔧 Доступные эндпоинты:");
    println!("   • Health Check: GET /api/health");
    println!("   • Версия API: GET /api/version");
    println!("   • Список эндпоинтов: GET /api/endpoints");
    println!("   • Админ-панель: /api/admin/*");
    println!("   • Преподаватели: /api/teacher/*");
    println!("   • Студенты: /api/student/*");
    
    axum::serve(listener, app)
        .await
        .expect("Ошибка при запуске сервера");
    
    Ok(())
}