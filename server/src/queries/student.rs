// Запросы для студентов
use sqlx::{PgPool, FromRow};
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct StudentCurriculum {
    pub subject_name: String,
    pub description: Option<String>,
    pub semester: i32,
    pub assessment_type: String,
    pub mark_status: String,
    pub teacher_name: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GroupMate {
    pub student_id: i32,
    pub full_name: String,
    pub birthday: Option<NaiveDate>,
    pub age: Option<i32>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct StudentMark {
    pub subject_name: String,
    pub semester: i32,
    pub assessment_type: String,
    pub teacher_name: String,
    pub mark_id: i32,
    pub mark_value: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CurrentSemesterSubject {
    pub id: i32,
    pub subject_name: String,
    pub description: Option<String>,
    pub assessment_type: String,
    pub teacher_name: String,
    pub completion_status: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SpecialityInfo {
    pub code: String,
    pub speciality_name: String,
    pub description: Option<String>,
    pub degree_name: Option<String>,
    pub current_course: i32,
    pub unit_name: String,
    pub university_name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct StudentSubject {
    pub subject_name: String,
    pub teacher_name: String,
    pub term: i32,
    pub clarification: String,
}

// INNER JOIN - Получение предметов студента с преподавателями
pub async fn get_student_subjects(
    pool: &PgPool,
    student_id: i32
) -> Result<Vec<StudentSubject>, sqlx::Error> {
    let query = r#"
        SELECT 
            sub.name as subject_name,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as teacher_name,
            ps.term,
            c.name as clarification
        FROM students s
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN curriculums cur ON cur.id = g.curriculum
        INNER JOIN planned_subjects ps ON ps.curriculum = cur.id
        INNER JOIN subjects sub ON sub.id = ps.subject
        INNER JOIN teaching_load tl ON tl.planned_subject = ps.id
        INNER JOIN teachers t ON t.id = tl.teacher
        INNER JOIN people p ON p.id = t.person
        INNER JOIN clarifications c ON c.id = ps.clarification
        WHERE s.id = $1
        ORDER BY ps.term, sub.name
    "#;
    
    sqlx::query_as::<_, StudentSubject>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Информация о группе и одногруппниках
pub async fn get_student_group_info(
    pool: &PgPool,
    student_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            g.course,
            spec.name as speciality_name,
            spec.code as speciality_code,
            d.name as degree_name,
            COUNT(classmates.id) as total_students,
            CONCAT(current_student.surname, ' ', current_student.name) as student_name,
            uni.name as university_name,
            unit.name as unit_name
        FROM students s
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        INNER JOIN degrees d ON d.id = spec.degree
        INNER JOIN people current_student ON current_student.id = s.person
        INNER JOIN units unit ON unit.id = g.affilation
        INNER JOIN universities uni ON uni.id = unit.university
        LEFT JOIN students classmates ON classmates."group" = g.id
        WHERE s.id = $1
        GROUP BY g.course, spec.name, spec.code, d.name, current_student.surname, 
                 current_student.name, uni.name, unit.name
    "#;
    
    let rows = sqlx::query(query)
        .bind(student_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "course": row.get::<i32, _>("course"),
                "speciality_name": row.get::<String, _>("speciality_name"),
                "speciality_code": row.get::<String, _>("speciality_code"),
                "degree_name": row.get::<Option<String>, _>("degree_name"),
                "total_students": row.get::<i64, _>("total_students"),
                "student_name": row.get::<String, _>("student_name"),
                "university_name": row.get::<String, _>("university_name"),
                "unit_name": row.get::<String, _>("unit_name")
            })
        })
        .collect();
    
    Ok(result)
}

// История оценок студента с EXISTS и CASE
pub async fn get_student_marks_history(
    pool: &PgPool,
    student_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            sub.name as subject_name,
            CONCAT(tp.surname, ' ', tp.name) as teacher_name,
            ps.term,
            c.name as clarification_type,
            CASE 
                WHEN m.id IS NOT NULL THEN 'Есть оценка'
                ELSE 'Оценка не выставлена'
            END as mark_status,
            CASE 
                WHEN EXISTS(
                    SELECT 1 FROM teaching_load tl2 
                    WHERE tl2.planned_subject = ps.id
                ) THEN 'Преподаватель назначен'
                ELSE 'Преподаватель не назначен'
            END as teacher_assigned,
            CASE 
                WHEN ps.term <= 4 THEN 'Начальные курсы'
                WHEN ps.term <= 6 THEN 'Средние курсы'
                ELSE 'Старшие курсы'
            END as study_period
        FROM students s
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN curriculums cur ON cur.id = g.curriculum
        INNER JOIN planned_subjects ps ON ps.curriculum = cur.id
        INNER JOIN subjects sub ON sub.id = ps.subject
        INNER JOIN clarifications c ON c.id = ps.clarification
        LEFT JOIN marks m ON m.student = s.id AND m.subject = ps.id
        LEFT JOIN teachers t ON t.id = m.teacher
        LEFT JOIN people tp ON tp.id = t.person
        WHERE s.id = $1
        ORDER BY ps.term, sub.name
    "#;
    
    let rows = sqlx::query(query)
        .bind(student_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "subject_name": row.get::<String, _>("subject_name"),
                "teacher_name": row.get::<Option<String>, _>("teacher_name"),
                "term": row.get::<i32, _>("term"),
                "clarification_type": row.get::<String, _>("clarification_type"),
                "mark_status": row.get::<String, _>("mark_status"),
                "teacher_assigned": row.get::<String, _>("teacher_assigned"),
                "study_period": row.get::<String, _>("study_period")
            })
        })
        .collect();
    
    Ok(result)
}

// Функции даты и времени - анализ возраста студента
pub async fn get_student_age_analysis(
    pool: &PgPool,
    student_id: i32
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name,
            p.birthday,
            CURRENT_DATE as today,
            CURRENT_TIMESTAMP as report_time,
            NOW()::date as current_date_only,
            CURRENT_TIME as current_time_now,
            LOCALTIMESTAMP as local_timestamp,
            EXTRACT(YEAR FROM CURRENT_DATE) as current_year,
            DATE_PART('year', p.birthday) as birth_year,
            AGE(CURRENT_DATE, p.birthday) as exact_age,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday))::INT as years_old,
            EXTRACT(MONTH FROM AGE(CURRENT_DATE, p.birthday))::INT as months_old,
            EXTRACT(DAY FROM AGE(CURRENT_DATE, p.birthday))::INT as days_old,
            DATE_PART('doy', CURRENT_DATE) as day_of_year,
            EXTRACT(DOW FROM CURRENT_DATE) as day_of_week
        FROM students s
        INNER JOIN people p ON p.id = s.person
        WHERE s.id = $1 AND p.birthday IS NOT NULL
    "#;
    
    let row = sqlx::query(query)
        .bind(student_id)
        .fetch_optional(pool)
        .await?;
    
    if let Some(row) = row {
        Ok(Some(serde_json::json!({
            "full_name": row.get::<String, _>("full_name"),
            "birthday": row.get::<NaiveDate, _>("birthday"),
            "today": row.get::<NaiveDate, _>("today"),
            "report_time": row.get::<DateTime<Utc>, _>("report_time"),
            "current_date_only": row.get::<NaiveDate, _>("current_date_only"),
            "local_timestamp": row.get::<chrono::NaiveDateTime, _>("local_timestamp"),
            "current_year": row.get::<f64, _>("current_year"),
            "birth_year": row.get::<Option<f64>, _>("birth_year"),
            "exact_age": row.get::<sqlx::postgres::types::PgInterval, _>("exact_age").to_string(),
            "years_old": row.get::<i32, _>("years_old"),
            "months_old": row.get::<i32, _>("months_old"),
            "days_old": row.get::<i32, _>("days_old"),
            "day_of_year": row.get::<f64, _>("day_of_year"),
            "day_of_week": row.get::<f64, _>("day_of_week")
        })))
    } else {
        Ok(None)
    }
}

// Агрегатные функции с GROUP BY - статистика по семестрам
pub async fn get_student_semester_statistics(
    pool: &PgPool,
    student_id: i32
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            ps.term as semester,
            COUNT(DISTINCT sub.id) as total_subjects,
            COUNT(DISTINCT m.id) as completed_subjects,
            COUNT(DISTINCT CASE WHEN c.name = 'Экзамен' THEN sub.id END) as exams_count,
            COUNT(DISTINCT CASE WHEN c.name = 'Зачет' THEN sub.id END) as tests_count,
            AVG(LENGTH(sub.name)) as avg_subject_name_length,
            MIN(LENGTH(sub.name)) as min_subject_name_length,
            MAX(LENGTH(sub.name)) as max_subject_name_length,
            SUM(CASE WHEN m.id IS NOT NULL THEN 1 ELSE 0 END) as marks_received
        FROM students s
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN curriculums cur ON cur.id = g.curriculum
        INNER JOIN planned_subjects ps ON ps.curriculum = cur.id
        INNER JOIN subjects sub ON sub.id = ps.subject
        INNER JOIN clarifications c ON c.id = ps.clarification
        LEFT JOIN marks m ON m.student = s.id AND m.subject = ps.id
        WHERE s.id = $1
        GROUP BY ps.term
        HAVING COUNT(DISTINCT sub.id) > 0
        ORDER BY ps.term
    "#;
    
    let rows = sqlx::query(query)
        .bind(student_id)
        .fetch_all(pool)
        .await?;
    
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "semester": row.get::<i32, _>("semester"),
                "total_subjects": row.get::<i64, _>("total_subjects"),
                "completed_subjects": row.get::<i64, _>("completed_subjects"),
                "exams_count": row.get::<i64, _>("exams_count"),
                "tests_count": row.get::<i64, _>("tests_count"),
                "avg_subject_name_length": row.get::<Option<f64>, _>("avg_subject_name_length"),
                "min_subject_name_length": row.get::<Option<i32>, _>("min_subject_name_length"),
                "max_subject_name_length": row.get::<Option<i32>, _>("max_subject_name_length"),
                "marks_received": row.get::<i64, _>("marks_received")
            })
        })
        .collect();
    
    Ok(result)
}

// Строковые функции - форматирование информации о студенте
pub async fn get_formatted_student_info(
    pool: &PgPool,
    student_id: i32
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.id,
            CONCAT(p.surname, ' ', p.name, 
                CASE WHEN p.patronymic IS NOT NULL THEN ' ' || p.patronymic ELSE '' END) as full_name,
            UPPER(CONCAT(p.surname, ' ', p.name)) as upper_name,
            LOWER(CONCAT(p.surname, ' ', p.name)) as lower_name,
            LENGTH(CONCAT(p.surname, p.name)) as name_total_length,
            SUBSTRING(p.surname FROM 1 FOR 3) as surname_prefix,
            OVERLAY(p.name PLACING '***' FROM 2 FOR 2) as masked_name,
            REPLACE(p.surname, 'ов', 'ОВ') as surname_replaced,
            BTRIM(CONCAT('  ', p.name, '  ')) as trimmed_name,
            LTRIM(CONCAT('  ', p.surname)) as left_trimmed_surname,
            STRPOS(LOWER(p.surname), 'а') as position_of_a,
            CHR(65) as chr_example,
            spec.name as speciality,
            g.course
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE s.id = $1
    "#;
    
    let row = sqlx::query(query)
        .bind(student_id)
        .fetch_optional(pool)
        .await?;
    
    if let Some(row) = row {
        Ok(Some(serde_json::json!({
            "id": row.get::<i32, _>("id"),
            "full_name": row.get::<String, _>("full_name"),
            "upper_name": row.get::<String, _>("upper_name"),
            "lower_name": row.get::<String, _>("lower_name"),
            "name_total_length": row.get::<i32, _>("name_total_length"),
            "surname_prefix": row.get::<String, _>("surname_prefix"),
            "masked_name": row.get::<String, _>("masked_name"),
            "surname_replaced": row.get::<String, _>("surname_replaced"),
            "trimmed_name": row.get::<String, _>("trimmed_name"),
            "left_trimmed_surname": row.get::<String, _>("left_trimmed_surname"),
            "position_of_a": row.get::<i32, _>("position_of_a"),
            "chr_example": row.get::<String, _>("chr_example"),
            "speciality": row.get::<String, _>("speciality"),
            "course": row.get::<i32, _>("course")
        })))
    } else {
        Ok(None)
    }
}

// COALESCE, NULLIF, GREATEST, LEAST - работа с NULL значениями
pub async fn get_student_data_with_nulls(
    pool: &PgPool,
    student_id: i32
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.id,
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
            g.course,
            spec.name as speciality
        FROM students s
        INNER JOIN people p ON p.id = s.person
        INNER JOIN groups g ON g.id = s."group"
        INNER JOIN specialities spec ON spec.id = g.speciality
        WHERE s.id = $1
    "#;
    
    let row = sqlx::query(query)
        .bind(student_id)
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
            "course": row.get::<i32, _>("course"),
            "speciality": row.get::<String, _>("speciality")
        })))
    } else {
        Ok(None)
    }
}

// Оригинальные функции (сохраняем для совместимости)

// Учебный план студента
pub async fn get_my_curriculum(pool: &PgPool, student_id: i32) -> Result<Vec<StudentCurriculum>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            s.description,
            ps.term as semester,
            cl.name as assessment_type,
            CASE 
                WHEN m.id IS NOT NULL THEN 'Оценка получена'
                ELSE 'Оценка не получена'
            END as mark_status,
            CONCAT(p.surname, ' ', p.name) as teacher_name
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject
        LEFT JOIN teachers t ON m.teacher = t.id
        LEFT JOIN people p ON t.person = p.id
        WHERE st.id = $1
        ORDER BY ps.term, s.name
    "#;
    
    sqlx::query_as::<_, StudentCurriculum>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Одногруппники студента
pub async fn get_my_group_mates(pool: &PgPool, student_id: i32) -> Result<Vec<GroupMate>, sqlx::Error> {
    let query = r#"
        SELECT 
            s2.id as student_id,
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as full_name,
            p.birthday,
            EXTRACT(YEAR FROM AGE(CURRENT_DATE, p.birthday))::INT as age
        FROM students s1
        INNER JOIN students s2 ON s1."group" = s2."group" AND s1.id != s2.id
        INNER JOIN people p ON s2.person = p.id
        WHERE s1.id = $1
        ORDER BY p.surname, p.name
    "#;
    
    sqlx::query_as::<_, GroupMate>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Мои оценки
pub async fn get_my_marks(pool: &PgPool, student_id: i32) -> Result<Vec<StudentMark>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            ps.term as semester,
            cl.name as assessment_type,
            CONCAT(p.surname, ' ', p.name) as teacher_name,
            m.id as mark_id,
            'Получена' as mark_value
        FROM marks m
        INNER JOIN planned_subjects ps ON m.subject = ps.id
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        INNER JOIN teachers t ON m.teacher = t.id
        INNER JOIN people p ON t.person = p.id
        WHERE m.student = $1
        ORDER BY ps.term, s.name
    "#;
    
    sqlx::query_as::<_, StudentMark>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Предметы текущего семестра
pub async fn get_current_semester_subjects(
    pool: &PgPool,
    student_id: i32, 
    current_term: i32
) -> Result<Vec<CurrentSemesterSubject>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.id,
            s.name as subject_name,
            s.description,
            cl.name as assessment_type,
            CASE 
                WHEN tl.teacher IS NOT NULL THEN CONCAT(p.surname, ' ', p.name)
                ELSE 'Преподаватель не назначен'
            END as teacher_name,
            CASE 
                WHEN m.id IS NOT NULL THEN 'Оценка получена'
                ELSE 'Оценка не получена'
            END as completion_status
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        LEFT JOIN teaching_load tl ON ps.id = tl.planned_subject
        LEFT JOIN teachers t ON tl.teacher = t.id
        LEFT JOIN people p ON t.person = p.id
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject
        WHERE st.id = $1 AND ps.term = $2
        ORDER BY s.name
    "#;
    
    sqlx::query_as::<_, CurrentSemesterSubject>(query)
        .bind(student_id)
        .bind(current_term)
        .fetch_all(pool)
        .await
}

// Информация о специальности студента
pub async fn get_speciality_info(pool: &PgPool, student_id: i32) -> Result<Option<SpecialityInfo>, sqlx::Error> {
    let query = r#"
        SELECT 
            sp.code,
            sp.name as speciality_name,
            sp.description,
            d.name as degree_name,
            g.course as current_course,
            u.name as unit_name,
            un.name as university_name
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN specialities sp ON g.speciality = sp.id
        INNER JOIN degrees d ON sp.degree = d.id
        INNER JOIN units u ON g.affilation = u.id
        INNER JOIN universities un ON u.university = un.id
        WHERE st.id = $1
    "#;
    
    sqlx::query_as::<_, SpecialityInfo>(query)
        .bind(student_id)
        .fetch_optional(pool)
        .await
}

// Прогресс обучения (статистика оценок)
pub async fn get_study_progress(
    pool: &PgPool, 
    student_id: i32
) -> Result<Vec<(i32, i64, i64, f64)>, sqlx::Error> {
    let query = r#"
        SELECT 
            ps.term as semester,
            COUNT(*) as total_subjects,
            COUNT(m.id) as completed_subjects,
            ROUND((COUNT(m.id)::FLOAT / COUNT(*)::FLOAT * 100), 2) as completion_percentage
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject
        WHERE st.id = $1
        GROUP BY ps.term
        ORDER BY ps.term
    "#;
    
    sqlx::query_as::<_, (i32, i64, i64, f64)>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Получить список преподавателей студента
pub async fn get_my_teachers(
    pool: &PgPool, 
    student_id: i32
) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let query = r#"
        SELECT DISTINCT
            CONCAT(p.surname, ' ', p.name, COALESCE(' ' || p.patronymic, '')) as teacher_name,
            s.name as subject_name,
            u.name as department_name
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN teaching_load tl ON ps.id = tl.planned_subject
        INNER JOIN teachers t ON tl.teacher = t.id
        INNER JOIN people p ON t.person = p.id
        INNER JOIN units u ON t.work = u.id
        WHERE st.id = $1
        ORDER BY teacher_name, subject_name
    "#;
    
    sqlx::query_as::<_, (String, String, String)>(query)
        .bind(student_id)
        .fetch_all(pool)
        .await
}

// Получить долги (несданные предметы)
pub async fn get_my_debts(
    pool: &PgPool, 
    student_id: i32,
    current_term: i32
) -> Result<Vec<(String, i32, String, String)>, sqlx::Error> {
    let query = r#"
        SELECT 
            s.name as subject_name,
            ps.term as semester,
            cl.name as assessment_type,
            COALESCE(CONCAT(p.surname, ' ', p.name), 'Преподаватель не назначен') as teacher_name
        FROM students st
        INNER JOIN groups g ON st."group" = g.id
        INNER JOIN curriculums c ON g.curriculum = c.id
        INNER JOIN planned_subjects ps ON c.id = ps.curriculum AND ps.term < $2
        INNER JOIN subjects s ON ps.subject = s.id
        INNER JOIN clarifications cl ON ps.clarification = cl.id
        LEFT JOIN marks m ON st.id = m.student AND ps.id = m.subject
        LEFT JOIN teaching_load tl ON ps.id = tl.planned_subject
        LEFT JOIN teachers t ON tl.teacher = t.id
        LEFT JOIN people p ON t.person = p.id
        WHERE st.id = $1 AND m.id IS NULL
        ORDER BY ps.term DESC, s.name
    "#;
    
    sqlx::query_as::<_, (String, i32, String, String)>(query)
        .bind(student_id)
        .bind(current_term)
        .fetch_all(pool)
        .await
}