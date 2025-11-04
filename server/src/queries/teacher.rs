// Запросы для преподавателей
use sqlx::{PgPool, FromRow, Row};
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TeachingLoad {
    pub subject_name: String,
    pub semester: i32,
    pub assessment_type: String,
    pub speciality_name: String,
    pub course: i32,
    pub student_count: i64,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TeacherStudent {
    pub student_id: i32,
    pub student_name: String,
    pub course: i32,
    pub speciality_name: String,
    pub subject_name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct StudentMark {
    pub student_id: i32,
    pub student_name: String,
    pub mark_id: Option<i32>,
    pub mark_status: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TeacherSubject {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub group_count: i64,
    pub specialities: Option<String>,
}

// Получить учебную нагрузку преподавателя
pub async fn get_my_teaching_load(pool: &PgPool, teacher_id: i32) -> Result<Vec<TeachingLoad>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            ps.term as semester,
            cl.name as assessment_type,
            sp.name as speciality_name,
            g.course,
            COUNT(st.id) as student_count
        FROM teaching_load tl
        INNER JOIN planned_subjects ps ON tl.planned_subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        INNER JOIN curriculums c ON ps.curriculum = c.id
        INNER JOIN groups g ON c.id = g.curriculum
        INNER JOIN specialities sp ON g.speciality = sp.id
        LEFT JOIN students st ON g.id = st."group"
        WHERE tl.teacher = $1
        GROUP BY s.name, ps.term, cl.name, sp.name, g.course, ps.id
        ORDER BY ps.term, s.name
    "#;
    
    sqlx::query_as::<_, TeachingLoad>(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await
}

// RIGHT JOIN - Все предметы и их возможные преподаватели
pub async fn get_subjects_with_teachers(
    pool: &PgPool, 
    teacher_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
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
            COALESCE(c.name, 'Форма контроля не указана') as clarification_type,
            CASE 
                WHEN t.id = $1 THEN 'Мой предмет'
                WHEN t.id IS NOT NULL THEN 'Чужой предмет'
                ELSE 'Преподаватель не назначен'
            END as assignment_status
        FROM teaching_load tl
        RIGHT JOIN planned_subjects ps ON ps.id = tl.planned_subject
        RIGHT JOIN subjects s ON s.id = ps.subject
        LEFT JOIN teachers t ON t.id = tl.teacher
        LEFT JOIN people p ON p.id = t.person
        LEFT JOIN clarifications c ON c.id = ps.clarification
        WHERE t.id = $1 OR t.id IS NULL OR ps.id IN (
            SELECT ps2.id FROM planned_subjects ps2 
            INNER JOIN teaching_load tl2 ON tl2.planned_subject = ps2.id 
            WHERE tl2.teacher = $1
        )
        ORDER BY s.name, ps.term NULLS LAST
    "#;
    
    let rows = sqlx::query(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "subject_name": row.get::<String, _>("subject_name"),
                "description": row.get::<String, _>("description"),
                "teacher_name": row.get::<String, _>("teacher_name"),
                "term": row.get::<i32, _>("term"),
                "clarification_type": row.get::<String, _>("clarification_type"),
                "assignment_status": row.get::<String, _>("assignment_status")
            })
        })
        .collect();
    
    Ok(result)
}

// INTERSECT - Общие студенты у разных преподавателей
pub async fn get_common_students(
    pool: &PgPool,
    teacher1_id: i32,
    teacher2_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            spec.name as speciality
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        INNER JOIN marks m1 ON m1.student = s.id
        WHERE m1.teacher = $1
        
        INTERSECT
        
        SELECT DISTINCT
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            spec.name as speciality
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        INNER JOIN marks m2 ON m2.student = s.id
        WHERE m2.teacher = $2
        
        ORDER BY student_name
    "#;
    
    let rows = sqlx::query(query)
        .bind(teacher1_id)
        .bind(teacher2_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "student_name": row.get::<String, _>("student_name"),
                "course": row.get::<i32, _>("course"),
                "speciality": row.get::<String, _>("speciality")
            })
        })
        .collect();
    
    Ok(result)
}

// EXCEPT - Студенты без оценок у конкретного преподавателя
pub async fn get_students_without_marks(
    pool: &PgPool,
    teacher_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            spec.name as speciality
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE EXISTS(
            SELECT 1 FROM teaching_load tl
            INNER JOIN planned_subjects ps ON ps.id = tl.planned_subject
            INNER JOIN curriculums c ON c.id = ps.curriculum
            WHERE tl.teacher = $1 AND c.id = g.curriculum
        )
        
        EXCEPT
        
        SELECT DISTINCT
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            spec.name as speciality
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        INNER JOIN marks m ON m.student = s.id
        WHERE m.teacher = $1
        
        ORDER BY student_name
    "#;
    
    let rows = sqlx::query(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "student_name": row.get::<String, _>("student_name"),
                "course": row.get::<i32, _>("course"),
                "speciality": row.get::<String, _>("speciality")
            })
        })
        .collect();
    
    Ok(result)
}

// IN, ANY, BETWEEN - Фильтрация студентов по критериям
pub async fn get_students_by_criteria(
    pool: &PgPool,
    teacher_id: i32,
    courses: Vec<i32>,
    min_age: i32,
    max_age: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as student_name,
            g.course,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday))::INT as age,
            spec.name as speciality,
            CASE 
                WHEN g.course = ANY($2::int[]) THEN 'Подходящий курс'
                ELSE 'Не подходящий курс'
            END as course_match,
            CASE 
                WHEN EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday)) BETWEEN $3 AND $4 
                THEN 'Подходящий возраст'
                ELSE 'Не подходящий возраст'
            END as age_match
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE EXISTS(
            SELECT 1 FROM marks m 
            WHERE m.student = s.id AND m.teacher = $1
        )
        AND g.course = ANY($2::int[])
        AND EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday)) BETWEEN $3 AND $4
        AND p.birthday IS NOT NULL
        ORDER BY g.course, student_name
    "#;
    
    let rows = sqlx::query(query)
        .bind(teacher_id)
        .bind(courses)
        .bind(min_age)
        .bind(max_age)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "student_name": row.get::<String, _>("student_name"),
                "course": row.get::<i32, _>("course"),
                "age": row.get::<Option<i32>, _>("age"),
                "speciality": row.get::<String, _>("speciality"),
                "course_match": row.get::<String, _>("course_match"),
                "age_match": row.get::<String, _>("age_match")
            })
        })
        .collect();
    
    Ok(result)
}

// LIKE и ILIKE - Поиск по паттернам с функциями для работы со строками
pub async fn search_subjects_and_students(
    pool: &PgPool,
    search_pattern: &str
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            'Предмет' as type,
            s.name as item_name,
            COALESCE(s.description, '') as description,
            0 as course,
            LENGTH(s.name) as name_length,
            UPPER(s.name) as upper_name,
            LOWER(s.name) as lower_name,
            SUBSTRING(s.name FROM 1 FOR 10) as name_preview,
            POSITION(LOWER($1) IN LOWER(s.name))::INT as match_position
        FROM subjects s
        WHERE s.name ILIKE $1
           OR s.description ILIKE $1
        
        UNION ALL
        
        SELECT 
            'Студент' as type,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as item_name,
            spec.name as description,
            g.course,
            LENGTH(CONCAT(p.surname, ' ', p.name)) as name_length,
            UPPER(CONCAT(p.surname, ' ', p.name)) as upper_name,
            LOWER(CONCAT(p.surname, ' ', p.name)) as lower_name,
            BTRIM(CONCAT('  ', p.surname, '  ')) as name_preview,
            STRPOS(LOWER(CONCAT(p.surname, ' ', p.name, ' ', COALESCE(p.patronymic, ''))), LOWER($1))::INT as match_position
        FROM students st
        INNER JOIN people p ON p.id = st.person
        INNER JOIN groups g ON g.id = st."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE CONCAT(p.surname, ' ', p.name, ' ', COALESCE(p.patronymic, '')) ILIKE $1
        
        ORDER BY type, item_name
    "#;
    
    let pattern = format!("%{}%", search_pattern);
    let rows = sqlx::query(query)
        .bind(pattern)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "type": row.get::<String, _>("type"),
                "item_name": row.get::<String, _>("item_name"),
                "description": row.get::<String, _>("description"),
                "course": row.get::<i32, _>("course"),
                "name_length": row.get::<i32, _>("name_length"),
                "upper_name": row.get::<String, _>("upper_name"),
                "lower_name": row.get::<String, _>("lower_name"),
                "name_preview": row.get::<String, _>("name_preview"),
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
            ) as child_head,
            parent.id as parent_id,
            child.id as child_id
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
                "child_head": row.get::<String, _>("child_head"),
                "parent_id": row.get::<i32, _>("parent_id"),
                "child_id": row.get::<i32, _>("child_id")
            })
        })
        .collect();
    
    Ok(result)
}

// Функции преобразования типов и работы с NULL
pub async fn get_formatted_teacher_info(
    pool: &PgPool,
    teacher_id: i32
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            t.id,
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
            ) as min_document_number,
            uni.name as university_name,
            unit.name as unit_name
        FROM teachers t
        INNER JOIN people p ON p.id = t.person
        INNER JOIN units unit ON unit.id = t.work
        INNER JOIN universities uni ON uni.id = unit.university
        WHERE t.id = $1
    "#;
    
    let row = sqlx::query(query)
        .bind(teacher_id)
        .fetch_optional(pool)
        .await?;
    
    if let Some(row) = row {
        Ok(Some(serde_json::json!({
            "id": row.get::<i32, _>("id"),
            "full_name": row.get::<String, _>("full_name"),
            "snils_info": row.get::<String, _>("snils_info"),
            "inn_info": row.get::<String, _>("inn_info"),
            "birth_date": row.get::<Option<String>, _>("birth_date"),
            "birth_date_alt": row.get::<Option<String>, _>("birth_date_alt"),
            "clean_patronymic": row.get::<Option<String>, _>("clean_patronymic"),
            "max_name_length": row.get::<i32, _>("max_name_length"),
            "min_document_number": row.get::<i64, _>("min_document_number"),
            "university_name": row.get::<String, _>("university_name"),
            "unit_name": row.get::<String, _>("unit_name")
        })))
    } else {
        Ok(None)
    }
}

// Функции даты и времени - Анализ активности преподавателя
pub async fn get_teacher_activity_timeline(
    pool: &PgPool,
    teacher_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            CURRENT_TIMESTAMP as report_time,
            NOW()::date as current_date_only,
            CURRENT_DATE as today,
            CURRENT_TIME as current_time_now,
            LOCALTIMESTAMP as local_timestamp,
            EXTRACT(YEAR FROM CURRENT_DATE) as current_year,
            DATE_PART('month', CURRENT_DATE) as current_month,
            EXTRACT(DOW FROM CURRENT_DATE) as day_of_week,
            AGE(CURRENT_DATE, p.birthday) as teacher_age,
            DATE_PART('year', AGE(CURRENT_DATE, p.birthday)) as years_old,
            CONCAT(p.surname, ' ', p.name) as teacher_name,
            COUNT(DISTINCT tl.id) as subjects_taught,
            COUNT(DISTINCT m.id) as marks_given
        FROM teachers t
        INNER JOIN people p ON p.id = t.person
        LEFT JOIN teaching_load tl ON tl.teacher = t.id
        LEFT JOIN marks m ON m.teacher = t.id
        WHERE t.id = $1
          AND p.birthday IS NOT NULL
        GROUP BY t.id, p.surname, p.name, p.birthday
    "#;
    
    let rows = sqlx::query(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "report_time": row.get::<chrono::DateTime<chrono::Utc>, _>("report_time"),
                "current_date_only": row.get::<chrono::NaiveDate, _>("current_date_only"),
                "today": row.get::<chrono::NaiveDate, _>("today"),
                "local_timestamp": row.get::<chrono::NaiveDateTime, _>("local_timestamp"),
                "current_year": row.get::<f64, _>("current_year"),
                "current_month": row.get::<f64, _>("current_month"),
                "day_of_week": row.get::<f64, _>("day_of_week"),
                "teacher_age": row.get::<sqlx::postgres::types::PgInterval, _>("teacher_age").to_string(),
                "years_old": row.get::<f64, _>("years_old"),
                "teacher_name": row.get::<String, _>("teacher_name"),
                "subjects_taught": row.get::<i64, _>("subjects_taught"),
                "marks_given": row.get::<i64, _>("marks_given")
            })
        })
        .collect();
    
    Ok(result)
}

// Получить всех студентов преподавателя (оригинальная функция)
pub async fn get_my_students(pool: &PgPool, teacher_id: i32) -> Result<Vec<TeacherStudent>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            st.id as student_id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as student_name,
            g.course,
            sp.name as speciality_name,
            s.name as subject_name
        FROM teaching_load tl
        INNER JOIN planned_subjects ps ON tl.planned_subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN curriculums c ON ps.curriculum = c.id
        INNER JOIN groups g ON c.id = g.curriculum
        INNER JOIN students st ON g.id = st."group"
        INNER JOIN people p ON st.person = p.id
        INNER JOIN specialities sp ON g.speciality = sp.id
        WHERE tl.teacher = $1
        ORDER BY g.course, sp.name, p.surname, p.name
    "#;
    
    sqlx::query_as::<_, TeacherStudent>(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await
}

// Оценки студентов по предмету (оригинальная функция)
pub async fn get_student_marks_by_subject(
    pool: &PgPool, 
    teacher_id: i32, 
    subject_id: i32
) -> Result<Vec<StudentMark>, sqlx::Error> {
    let query = r#"
        SELECT 
            st.id as student_id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as student_name,
            m.id as mark_id,
            CASE 
                WHEN m.id IS NOT NULL THEN 'Оценка выставлена'
                ELSE 'Оценка не выставлена'
            END as mark_status
        FROM students st
        INNER JOIN people p ON st.person = p.id
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum AND ps.subject = $2
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject AND m.teacher = $1
        WHERE EXISTS (
            SELECT 1 
            FROM teaching_load tl 
            WHERE tl.teacher = $1 AND tl.planned_subject = ps.id
        )
        ORDER BY p.surname, p.name
    "#;
    
    sqlx::query_as::<_, StudentMark>(query)
        .bind(teacher_id)
        .bind(subject_id)
        .fetch_all(pool)
        .await
}

// Дисциплины, которые преподает данный преподаватель (оригинальная функция)
pub async fn get_subjects_i_teach(pool: &PgPool, teacher_id: i32) -> Result<Vec<TeacherSubject>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            s.id,
            s.name,
            s.description,
            COUNT(DISTINCT g.id) as group_count,
            STRING_AGG(DISTINCT sp.name, ', ' ORDER BY sp.name) as specialities
        FROM teaching_load tl
        INNER JOIN planned_subjects ps ON tl.planned_subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN curriculums c ON ps.curriculum = c.id
        INNER JOIN groups g ON c.id = g.curriculum
        INNER JOIN specialities sp ON g.speciality = sp.id
        WHERE tl.teacher = $1
        GROUP BY s.id, s.name, s.description
        ORDER BY s.name
    "#;
    
    sqlx::query_as::<_, TeacherSubject>(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await
}

// Получить статистику по оценкам (оригинальная функция)
pub async fn get_teacher_marks_statistics(
    pool: &PgPool, 
    teacher_id: i32
) -> Result<Vec<(String, i64, i64)>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            COUNT(*) as total_students,
            COUNT(m.id) as graded_students
        FROM teaching_load tl
        INNER JOIN planned_subjects ps ON tl.planned_subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN curriculums c ON ps.curriculum = c.id
        INNER JOIN groups g ON c.id = g.curriculum
        INNER JOIN students st ON g.id = st."group"
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject AND m.teacher = $1
        WHERE tl.teacher = $1
        GROUP BY s.id, s.name
        ORDER BY s.name
    "#;
    
    sqlx::query_as::<_, (String, i64, i64)>(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await
}

// Получить график занятий (по семестрам) (оригинальная функция)
pub async fn get_teacher_schedule(
    pool: &PgPool, 
    teacher_id: i32
) -> Result<Vec<(i32, String, String, i32)>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            ps.term as semester,
            s.name as subject_name,
            cl.name as assessment_type,
            COUNT(DISTINCT g.id) as group_count
        FROM teaching_load tl
        INNER JOIN planned_subjects ps ON tl.planned_subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        INNER JOIN curriculums c ON ps.curriculum = c.id
        INNER JOIN groups g ON c.id = g.curriculum
        WHERE tl.teacher = $1
        GROUP BY ps.term, s.name, cl.name, ps.id
        ORDER BY ps.term, s.name
    "#;
    
    sqlx::query_as::<_, (i32, String, String, i32)>(query)
        .bind(teacher_id)
        .fetch_all(pool)
        .await
}
