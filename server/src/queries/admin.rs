// Административные запросы для SQLx
use sqlx::{PgPool, FromRow, Row};
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TeachingLoadStats {
    pub teacher_name: String,
    pub subject_count: i64,
    pub university: String,
    pub workload_status: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct StructuralUnit {
    pub unit_name: String,
    pub unit_type: String,
    pub head_name: Option<String>,
    pub children_count: i64,
    pub parent_unit: Option<String>,
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

// FULL JOIN - Полная нагрузка преподавателей со всеми предметами
pub async fn get_full_teaching_load(pool: &PgPool) -> Result<Vec<TeachingLoadStats>, sqlx::Error> {
    let query = r#"
        SELECT 
            COALESCE(
                CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END), 
                'Неназначенный преподаватель'
            ) as teacher_name,
            COUNT(DISTINCT s.id) as subject_count,
            COALESCE(uni.name, 'Неизвестный университет') as university,
            CASE 
                WHEN COUNT(DISTINCT s.id) > 5 THEN 'Высокая нагрузка'
                WHEN COUNT(DISTINCT s.id) BETWEEN 3 AND 5 THEN 'Средняя нагрузка'
                WHEN COUNT(DISTINCT s.id) > 0 THEN 'Низкая нагрузка'
                ELSE 'Без нагрузки'
            END as workload_status
        FROM subjects s
        FULL JOIN planned_subjects ps ON ps.subject = s.id
        FULL JOIN teaching_load tl ON tl.planned_subject = ps.id
        FULL JOIN teachers t ON t.id = tl.teacher
        FULL JOIN people p ON p.id = t.person
        FULL JOIN units un ON un.id = t.work
        FULL JOIN universities uni ON uni.id = un.university
        GROUP BY p.id, p.surname, p.name, p.patronymic, uni.name
        HAVING COUNT(DISTINCT s.id) > 0 OR p.id IS NULL
        ORDER BY subject_count DESC NULLS LAST
    "#;
    
    sqlx::query_as::<_, TeachingLoadStats>(query)
        .fetch_all(pool)
        .await
}

// CROSS JOIN LATERAL - Структурные подразделения с количеством дочерних элементов
pub async fn get_structural_hierarchy(pool: &PgPool) -> Result<Vec<StructuralUnit>, sqlx::Error> {
    let query = r#"
        SELECT 
            u.name as unit_name,
            ut.name as unit_type,
            CASE 
                WHEN p.id IS NOT NULL THEN 
                    CONCAT(p.surname, ' ', p.name, 
                    CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END)
                ELSE NULL 
            END as head_name,
            child_stats.children_count,
            parent_unit.name as parent_unit
        FROM units u
        INNER JOIN unit_types ut ON ut.id = u.type
        LEFT JOIN people p ON p.id = u.head
        LEFT JOIN units parent_unit ON parent_unit.id = u.parent
        CROSS JOIN LATERAL (
            SELECT COUNT(*) as children_count
            FROM units child 
            WHERE child.parent = u.id
        ) child_stats
        ORDER BY ut.name, children_count DESC
    "#;
    
    sqlx::query_as::<_, StructuralUnit>(query)
        .fetch_all(pool)
        .await
}

// RIGHT JOIN - Все предметы и их возможные преподаватели
pub async fn get_subjects_with_possible_teachers(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            COALESCE(s.description, 'Описание отсутствует') as description,
            COALESCE(
                CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END),
                'Преподаватель не назначен'
            ) as teacher_name,
            COALESCE(ps.term, 0) as term,
            COALESCE(c.name, 'Форма контроля не указана') as clarification_type
        FROM teaching_load tl
        RIGHT JOIN planned_subjects ps ON ps.id = tl.planned_subject
        RIGHT JOIN subjects s ON s.id = ps.subject
        LEFT JOIN teachers t ON t.id = tl.teacher
        LEFT JOIN people p ON p.id = t.person
        LEFT JOIN clarifications c ON c.id = ps.clarification
        ORDER BY s.name, ps.term NULLS LAST
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "subject_name": row.get::<String, _>("subject_name"),
                "description": row.get::<String, _>("description"),
                "teacher_name": row.get::<String, _>("teacher_name"),
                "term": row.get::<i32, _>("term"),
                "clarification_type": row.get::<String, _>("clarification_type")
            })
        })
        .collect();
    
    Ok(result)
}

// UNION ALL - Все люди в системе с их ролями
pub async fn get_teachers_and_students_union(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            p.id,
            p.surname,
            p.name,
            p.patronymic,
            'Студент' as role,
            s.id as role_id,
            g.course as additional_info
        FROM people p
        INNER JOIN students s ON s.person = p.id
        INNER JOIN groups g ON g.id = s."group"
        
        UNION ALL
        
        SELECT 
            p.id,
            p.surname,
            p.name,
            p.patronymic,
            'Преподаватель' as role,
            t.id as role_id,
            0 as additional_info
        FROM people p
        INNER JOIN teachers t ON t.person = p.id
        
        UNION ALL
        
        SELECT 
            p.id,
            p.surname,
            p.name,
            p.patronymic,
            'Ректор' as role,
            u.id as role_id,
            0 as additional_info
        FROM people p
        INNER JOIN universities u ON u.rector = p.id
        
        ORDER BY surname, name
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "surname": row.get::<String, _>("surname"),
                "name": row.get::<String, _>("name"),
                "patronymic": row.get::<Option<String>, _>("patronymic"),
                "role": row.get::<String, _>("role"),
                "role_id": row.get::<i32, _>("role_id"),
                "additional_info": row.get::<i32, _>("additional_info")
            })
        })
        .collect();
    
    Ok(result)
}

// INTERSECT - Общие люди в разных ролях
pub async fn get_people_multiple_roles(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            p.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name
        FROM people p
        INNER JOIN teachers t ON p.id = t.person
        
        INTERSECT
        
        SELECT DISTINCT
            p.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name
        FROM people p
        INNER JOIN units u ON p.id = u.head
        
        ORDER BY full_name
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "full_name": row.get::<String, _>("full_name")
            })
        })
        .collect();
    
    Ok(result)
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
        ORDER BY full_name
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

// EXISTS - Преподаватели с нагрузкой и проверка возраста
pub async fn get_teachers_workload_status(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            t.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name,
            CASE 
                WHEN EXISTS(SELECT 1 FROM teaching_load tl WHERE tl.teacher = t.id) 
                THEN 'Имеет нагрузку'
                ELSE 'Без нагрузки'
            END as workload_status,
            CASE 
                WHEN EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday)) BETWEEN 25 AND 35 THEN 'Молодой'
                WHEN EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday)) BETWEEN 36 AND 50 THEN 'Средний возраст'
                WHEN EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday)) > 50 THEN 'Опытный'
                ELSE 'Возраст не указан'
            END as age_category,
            uni.name as university_name
        FROM teachers t
        INNER JOIN people p ON p.id = t.person
        INNER JOIN units u ON u.id = t.work
        INNER JOIN universities uni ON uni.id = u.university
        WHERE p.birthday IS NOT NULL
        ORDER BY p.surname, p.name
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "full_name": row.get::<String, _>("full_name"),
                "workload_status": row.get::<String, _>("workload_status"),
                "age_category": row.get::<String, _>("age_category"),
                "university_name": row.get::<String, _>("university_name")
            })
        })
        .collect();
    
    Ok(result)
}

// IN, ALL, ANY, SOME - Поиск студентов по нескольким критериям
pub async fn get_students_by_multiple_criteria(
    pool: &PgPool,
    courses: Vec<i32>,
    speciality_ids: Vec<i32>
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            spec.name as speciality,
            CASE 
                WHEN g.course = ANY($1::int[]) THEN 'Подходящий курс'
                ELSE 'Не подходящий курс'
            END as course_match,
            CASE 
                WHEN spec.id = ANY($2::int[]) THEN 'Подходящая специальность'
                ELSE 'Не подходящая специальность'
            END as speciality_match
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE g.course = ANY($1::int[])
          AND spec.id = ANY($2::int[])
          AND EXISTS(SELECT 1 FROM marks m WHERE m.student = s.id)
        ORDER BY g.course, student_name
    "#;
    
    let rows = sqlx::query(query)
        .bind(courses)
        .bind(speciality_ids)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "student_name": row.get::<String, _>("student_name"),
                "course": row.get::<i32, _>("course"),
                "speciality": row.get::<String, _>("speciality"),
                "course_match": row.get::<String, _>("course_match"),
                "speciality_match": row.get::<String, _>("speciality_match")
            })
        })
        .collect();
    
    Ok(result)
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

// LIKE и ILIKE - Поиск людей по шаблону имени с функциями для работы со строками
pub async fn search_people_formatted(
    pool: &PgPool, 
    search_term: &str
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            id,
            UPPER(CONCAT(surname, ' ', name)) as display_name,
            LENGTH(CONCAT(surname, name)) as name_length,
            CASE 
                WHEN STRPOS(LOWER(CONCAT(surname, ' ', name, ' ', COALESCE(patronymic, ''))), $1) > 0 
                THEN 'Найдено'
                ELSE 'Не найдено'
            END as search_status,
            OVERLAY(surname PLACING '***' FROM 2 FOR 2) as masked_surname,
            SUBSTRING(name FROM 1 FOR 3) as name_prefix,
            REPLACE(LOWER(surname), 'а', 'А') as surname_modified,
            BTRIM(CONCAT('  ', name, '  ')) as trimmed_name,
            POSITION(LOWER($1) IN LOWER(CONCAT(surname, ' ', name, ' ', COALESCE(patronymic, ''))))::INT as match_position
        FROM people
        WHERE LOWER(CONCAT(surname, ' ', name, ' ', COALESCE(patronymic, ''))) ILIKE $1
        ORDER BY surname, name
        LIMIT 50
    "#;
    
    let search_pattern = format!("%{}%", search_term.to_lowercase());
    let rows = sqlx::query(query)
        .bind(search_pattern)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "display_name": row.get::<String, _>("display_name"),
                "name_length": row.get::<i32, _>("name_length"),
                "search_status": row.get::<String, _>("search_status"),
                "masked_surname": row.get::<String, _>("masked_surname"),
                "name_prefix": row.get::<String, _>("name_prefix"),
                "surname_modified": row.get::<String, _>("surname_modified"),
                "trimmed_name": row.get::<String, _>("trimmed_name"),
                "match_position": row.get::<i32, _>("match_position")
            })
        })
        .collect();
    
    Ok(result)
}

// Самосоединение - Иерархия структурных подразделений
pub async fn get_department_hierarchy(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            parent.name as parent_unit,
            child.name as child_unit,
            parent_type.name as parent_type,
            child_type.name as child_type,
            COALESCE(
                CONCAT(parent_head.surname, ' ', parent_head.name),
                'Руководитель не назначен'
            ) as parent_head,
            COALESCE(
                CONCAT(child_head.surname, ' ', child_head.name),
                'Руководитель не назначен'
            ) as child_head
        FROM units parent
        INNER JOIN units child ON child.parent = parent.id
        INNER JOIN unit_types parent_type ON parent_type.id = parent.type
        INNER JOIN unit_types child_type ON child_type.id = child.type
        LEFT JOIN people parent_head ON parent_head.id = parent.head
        LEFT JOIN people child_head ON child_head.id = child.head
        ORDER BY parent.name, child.name
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "parent_unit": row.get::<String, _>("parent_unit"),
                "child_unit": row.get::<String, _>("child_unit"),
                "parent_type": row.get::<String, _>("parent_type"),
                "child_type": row.get::<String, _>("child_type"),
                "parent_head": row.get::<String, _>("parent_head"),
                "child_head": row.get::<String, _>("child_head")
            })
        })
        .collect();
    
    Ok(result)
}

// Функции преобразования типов (CAST, ::, COALESCE, NULLIF, GREATEST, LEAST)
pub async fn get_formatted_person_data(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            p.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name,
            COALESCE(p.snils::text, 'СНИЛС не указан') as snils_info,
            COALESCE(p.inn::text, 'ИНН не указан') as inn_info,
            CAST(p.birthday AS text) as birth_date,
            p.birthday::text as birth_date_alt,
            NULLIF(BTRIM(p.patronymic), '') as clean_patronymic,
            GREATEST(
                LENGTH(p.surname), 
                LENGTH(p.name), 
                COALESCE(LENGTH(p.patronymic), 0)
            ) as max_name_length,
            LEAST(
                COALESCE(p.snils, 999999999999), 
                COALESCE(p.inn, 999999999999)
            ) as min_document_number
        FROM people p
        WHERE p.birthday IS NOT NULL
        ORDER BY p.surname, p.name
        LIMIT 100
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "full_name": row.get::<String, _>("full_name"),
                "snils_info": row.get::<String, _>("snils_info"),
                "inn_info": row.get::<String, _>("inn_info"),
                "birth_date": row.get::<Option<String>, _>("birth_date"),
                "birth_date_alt": row.get::<Option<String>, _>("birth_date_alt"),
                "clean_patronymic": row.get::<Option<String>, _>("clean_patronymic"),
                "max_name_length": row.get::<i32, _>("max_name_length"),
                "min_document_number": row.get::<i64, _>("min_document_number")
            })
        })
        .collect();
    
    Ok(result)
}

// Функции даты и времени - Анализ временных данных
pub async fn get_age_statistics(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            DATE_PART('year', birthday) as birth_year,
            COUNT(*) as people_count,
            AVG(EXTRACT(YEAR FROM AGE(CURRENT_DATE, birthday))) as average_age,
            MIN(AGE(CURRENT_DATE, birthday)) as min_age,
            MAX(AGE(CURRENT_DATE, birthday)) as max_age,
            CURRENT_TIMESTAMP as report_time,
            NOW()::date as current_date_only,
            CURRENT_DATE as today,
            CURRENT_TIME as current_time_now,
            LOCALTIMESTAMP as local_timestamp
        FROM people
        WHERE birthday IS NOT NULL
            AND birthday BETWEEN '1950-01-01' AND CURRENT_DATE
        GROUP BY DATE_PART('year', birthday)
        HAVING COUNT(*) >= 1
        ORDER BY birth_year DESC
        LIMIT 20
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "birth_year": row.get::<f64, _>("birth_year"),
                "people_count": row.get::<i64, _>("people_count"),
                "average_age": row.get::<Option<f64>, _>("average_age"),
                "min_age": row.get::<sqlx::postgres::types::PgInterval, _>("min_age").to_string(),
                "max_age": row.get::<sqlx::postgres::types::PgInterval, _>("max_age").to_string(),
                "report_time": row.get::<DateTime<Utc>, _>("report_time"),
                "current_date_only": row.get::<NaiveDate, _>("current_date_only"),
                "today": row.get::<NaiveDate, _>("today"),
                "local_timestamp": row.get::<chrono::NaiveDateTime, _>("local_timestamp")
            })
        })
        .collect();
    
    Ok(result)
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
        ORDER BY s.name
    "#;
    
    sqlx::query_as::<_, (i32, String, Option<String>)>(query)
        .fetch_all(pool)
        .await
}

// Агрегатные функции и группировка с HAVING - Статистика по специальностям
pub async fn get_speciality_statistics(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.code,
            s.name as speciality_name,
            d.name as degree_name,
            COUNT(g.id) as groups_count,
            COUNT(st.id) as students_count,
            AVG(g.course) as avg_course,
            MIN(g.course) as min_course,
            MAX(g.course) as max_course,
            SUM(CASE WHEN g.course >= 3 THEN 1 ELSE 0 END) as senior_groups
        FROM specialities s
        INNER JOIN degrees d ON d.id = s.degree
        LEFT JOIN groups g ON g.speciality = s.id
        LEFT JOIN students st ON st."group" = g.id
        GROUP BY s.id, s.code, s.name, d.name
        HAVING COUNT(g.id) > 0
        ORDER BY students_count DESC, speciality_name
    "#;
    
    let rows = sqlx::query(query).fetch_all(pool).await?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "code": row.get::<String, _>("code"),
                "speciality_name": row.get::<String, _>("speciality_name"),
                "degree_name": row.get::<Option<String>, _>("degree_name"),
                "groups_count": row.get::<i64, _>("groups_count"),
                "students_count": row.get::<i64, _>("students_count"),
                "avg_course": row.get::<Option<f64>, _>("avg_course"),
                "min_course": row.get::<Option<i32>, _>("min_course"),
                "max_course": row.get::<Option<i32>, _>("max_course"),
                "senior_groups": row.get::<i64, _>("senior_groups")
            })
        })
        .collect();
    
    Ok(result)
}

// Агрегатные функции с группировкой - Оригинальная статистика университетов
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
