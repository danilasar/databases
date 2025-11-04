// Запросы для студентов
use sqlx::{PgPool, FromRow};
use chrono::NaiveDate;
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
