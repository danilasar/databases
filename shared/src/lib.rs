use serde::{Deserialize, Serialize};
use chrono::{Date, NaiveDate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<i32>,
    pub surname: String,
    pub name: String,
    pub patronymic: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub snils: Option<i64>,
    pub inn: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct University {
    pub id: Option<i32>,
    pub name: String,
    pub full_name: String,
    pub rector: Option<i32>,
    pub rector_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitType {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: Option<i32>,
    pub name: String,
    pub university: i32,
    pub university_name: Option<String>,
    pub type_id: i32,
    pub type_name: Option<String>,
    pub parent: Option<i32>,
    pub parent_name: Option<String>,
    pub head: Option<i32>,
    pub head_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
    pub id: Option<i32>,
    pub person: i32,
    pub person_name: Option<String>,
    pub work: i32,
    pub work_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Degree {
    pub id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speciality {
    pub id: Option<i32>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub degree: i32,
    pub degree_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: Option<i32>,
    pub person: i32,
    pub person_name: Option<String>,
    pub group: i32,
    pub group_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Option<i32>,
    pub course: i32,
    pub affilation: i32,
    pub affilation_name: Option<String>,
    pub speciality: i32,
    pub speciality_name: Option<String>,
    pub curriculum: i32,
}

