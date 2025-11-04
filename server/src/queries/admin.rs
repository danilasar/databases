// Административные запросы для SQLx
use sqlx::{PgPool, FromRow};
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UniversityStructure {
    pub id: i32,
    pub unit_name: String,
    pub unit_type: String,
    pub university_name: String,
    pub head_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PersonWithRole {
    pub id: i32,
    pub full_name: String,
    pub birthday: Option<NaiveDate>,
    pub role_type: String,
    pub age: Option<i32>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UniversityStatistics {
    pub university_name: String,
    pub total_units: i64,
    pub total_teachers: i64,
    pub total_groups: i64,
    pub total_students: i64,
    pub oldest_person_birthday: Option<NaiveDate>,
    pub youngest_person_birthday: Option<NaiveDate>,
    pub average_age: Option<f64>,
    pub teacher_count: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PersonAge {
    pub id: i32,
    pub full_name: String,
    pub birthday: Option<NaiveDate>,
    pub age: i32,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct PersonSearch {
    pub id: i32,
    pub surname: String,
    pub name: String,
    pub patronymic: Option<String>,
    pub match_location: String,
    pub match_position: i32,
}

// INNER JOIN - Получить структуру университета с типами подразделений
pub async fn get_university_structure(pool: &PgPool) -> Result<Vec<UniversityStructure>, sqlx::Error> {
    let query = r#"
        SELECT 
            u.id,
            u.name as unit_name,
            ut.name as unit_type,
            un.name as university_name,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as head_name
        FROM units u
        INNER JOIN unit_types ut ON u.type = ut.id
        INNER JOIN universities un ON u.university = un.id
        INNER JOIN people p ON u.head = p.id
        ORDER BY un.name, ut.name, u.name
    "#;
    
    sqlx::query_as::<_, UniversityStructure>(query)
        .fetch_all(pool)
        .await
}

// LEFT JOIN - Получить всех людей с их возможными ролями
pub async fn get_all_people_with_roles(pool: &PgPool) -> Result<Vec<PersonWithRole>, sqlx::Error> {
    let query = r#"
        SELECT 
            p.id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name,
            p.birthday,
            CASE 
                WHEN t.id IS NOT NULL THEN 'Преподаватель'
                WHEN s.id IS NOT NULL THEN 'Студент'
                WHEN u.rector = p.id THEN 'Ректор'
                WHEN un.head = p.id THEN 'Руководитель подразделения'
                ELSE 'Обычный человек'
            END as role_type,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday))::INT as age
        FROM people p
        LEFT JOIN teachers t ON p.id = t.person
        LEFT JOIN students s ON p.id = s.person
        LEFT JOIN universities u ON p.id = u.rector
        LEFT JOIN units un ON p.id = un.head
        ORDER BY p.surname, p.name
    "#;
    
    sqlx::query_as::<_, PersonWithRole>(query)
        .fetch_all(pool)
        .await
}

// UNION - Объединение преподавателей и студентов
pub async fn get_teachers_and_students_union(pool: &PgPool) -> Result<Vec<PersonWithRole>, sqlx::Error> {
    let query = r#"
        SELECT 
            'Преподаватель' as person_type,
            p.id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name,
            u.name as workplace
        FROM teachers t
        INNER JOIN people p ON t.person = p.id
        INNER JOIN units u ON t.work = u.id
        UNION
        SELECT 
            'Студент' as person_type,
            p.id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name,
            CONCAT('Группа курс ', g.course, ', ', sp.name) as workplace
        FROM students s
        INNER JOIN people p ON s.person = p.id
        INNER JOIN groups g ON s."group" = g.id
        INNER JOIN specialities sp ON g.speciality = sp.id
        ORDER BY person_type, full_name
    "#;
    
    #[derive(Debug, FromRow, Serialize)]
    struct UnionResult {
        person_type: String,
        id: i32,
        full_name: String,
        workplace: String,
    }
    
    let results = sqlx::query_as::<_, UnionResult>(query)
        .fetch_all(pool)
        .await?;
    
    Ok(results.into_iter().map(|r| PersonWithRole {
        id: r.id,
        full_name: r.full_name,
        birthday: None,
        role_type: r.person_type,
        age: None,
    }).collect())
}

// EXCEPT - Люди, которые НЕ связаны с университетом
pub async fn get_people_not_in_university(pool: &PgPool) -> Result<Vec<PersonWithRole>, sqlx::Error> {
    let query = r#"
        SELECT 
            p.id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name
        FROM people p
        EXCEPT
        (
            SELECT 
                p.id,
                CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name
            FROM people p
            INNER JOIN teachers t ON p.id = t.person
            
            UNION
            
            SELECT 
                p.id,
                CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name
            FROM people p
            INNER JOIN students s ON p.id = s.person
            
            UNION
            
            SELECT 
                p.id,
                CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name
            FROM people p
            INNER JOIN universities u ON p.id = u.rector
            
            UNION
            
            SELECT 
                p.id,
                CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name
            FROM people p
            INNER JOIN units un ON p.id = un.head
        )
    "#;
    
    #[derive(Debug, FromRow)]
    struct SimpleResult {
        id: i32,
        full_name: String,
    }
    
    let results = sqlx::query_as::<_, SimpleResult>(query)
        .fetch_all(pool)
        .await?;
    
    Ok(results.into_iter().map(|r| PersonWithRole {
        id: r.id,
        full_name: r.full_name,
        birthday: None,
        role_type: "Не связан с университетом".to_string(),
        age: None,
    }).collect())
}

// EXISTS - Дисциплины, которые реально преподаются
pub async fn check_active_subjects(pool: &PgPool) -> Result<Vec<(i32, String, Option<String>)>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.id,
            s.name,
            s.description
        FROM subjects s
        WHERE EXISTS (
            SELECT 1 
            FROM planned_subjects ps 
            WHERE ps.subject = s.id
        )
    "#;
    
    sqlx::query_as::<_, (i32, String, Option<String>)>(query)
        .fetch_all(pool)
        .await
}

// BETWEEN - Люди в определенном возрастном диапазоне
pub async fn get_people_by_age_range(pool: &PgPool, min_age: i32, max_age: i32) -> Result<Vec<PersonAge>, sqlx::Error> {
    let query = r#"
        SELECT 
            id,
            CONCAT(surname, ' ', name, COALESCE(' ' || patronymic, '')) as full_name,
            birthday,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, birthday))::INT as age
        FROM people
        WHERE birthday IS NOT NULL 
        AND EXTRACT(YEAR FROM AGE(CURRENT_DATE, birthday)) BETWEEN $1 AND $2
        ORDER BY birthday DESC
    "#;
    
    sqlx::query_as::<_, PersonAge>(query)
        .bind(min_age)
        .bind(max_age)
        .fetch_all(pool)
        .await
}

// LIKE и ILIKE - Поиск людей по шаблону имени
pub async fn search_people_by_name(pool: &PgPool, search_pattern: &str) -> Result<Vec<PersonSearch>, sqlx::Error> {
    let query = r#"
        SELECT 
            id,
            surname,
            name,
            patronymic,
            CASE 
                WHEN surname ILIKE $1 THEN 'Найдено в фамилии'
                WHEN name ILIKE $1 THEN 'Найдено в имени'
                WHEN patronymic ILIKE $1 THEN 'Найдено в отчестве'
                ELSE 'Не найдено'
            END as match_location,
            POSITION(LOWER($1) IN LOWER(CONCAT(surname, ' ', name, ' ', COALESCE(patronymic, ''))))::INT as match_position
        FROM people
        WHERE surname ILIKE $1 
           OR name ILIKE $1 
           OR patronymic ILIKE $1
    "#;
    
    let pattern = format!("%{}%", search_pattern);
    sqlx::query_as::<_, PersonSearch>(query)
        .bind(&pattern)
        .fetch_all(pool)
        .await
}

// Агрегатные функции и группировка
pub async fn get_university_statistics(pool: &PgPool) -> Result<Vec<UniversityStatistics>, sqlx::Error> {
    let query = r#"
        SELECT 
            u.name as university_name,
            COUNT(un.id) as total_units,
            COUNT(t.id) as total_teachers,
            COUNT(DISTINCT g.id) as total_groups,
            COUNT(s.id) as total_students,
            MIN(p.birthday) as oldest_person_birthday,
            MAX(p.birthday) as youngest_person_birthday,
            AVG(EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday))) as average_age,
            SUM(CASE WHEN t.id IS NOT NULL THEN 1 ELSE 0 END) as teacher_count
        FROM universities u
        LEFT JOIN units un ON u.id = un.university
        LEFT JOIN teachers t ON un.id = t.work
        LEFT JOIN groups g ON un.id = g.affilation
        LEFT JOIN students s ON g.id = s."group"
        LEFT JOIN people p ON (t.person = p.id OR s.person = p.id)
        GROUP BY u.id, u.name
        HAVING COUNT(un.id) > 0
        ORDER BY total_students DESC
    "#;
    
    sqlx::query_as::<_, UniversityStatistics>(query)
        .fetch_all(pool)
        .await
}
