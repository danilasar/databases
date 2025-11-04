// Запросы для преподавателей
use sqlx::{PgPool, FromRow};
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

// Получить всех студентов преподавателя
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

// Оценки студентов по предмету
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

// Дисциплины, которые преподает данный преподаватель
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

// Получить статистику по оценкам
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

// Получить график занятий (по семестрам)
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
