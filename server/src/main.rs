// Основной файл сервера университетской базы данных
use axum::{
    extract::Extension,
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
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
    
    // Создаем основной роутер
    let app = Router::new()
        // Административные маршруты
        .route("/api/admin/university-structure", get(handlers::get_university_structure))
        .route("/api/admin/people-with-roles", get(handlers::get_all_people_with_roles))
        .route("/api/admin/university-statistics", get(handlers::get_university_statistics))
        
        // Маршруты для преподавателей
        .route("/api/teacher/:id/teaching-load", get(handlers::get_my_teaching_load))
        .route("/api/teacher/:id/students", get(handlers::get_my_students))
        
        // Маршруты для студентов
        .route("/api/student/:id/curriculum", get(handlers::get_my_curriculum))
        .route("/api/student/:id/marks", get(handlers::get_my_marks))
        
        // Общие маршруты
        .route("/api/health", get(health_check))
        .route("/api/version", get(get_version))
        
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
    
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Не удалось привязаться к адресу");
    
    println!("✅ Сервер успешно запущен!");
    
    axum::serve(listener, app)
        .await
        .expect("Ошибка при запуске сервера");
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_version() -> &'static str {
    "University Database API v0.1.0"
}