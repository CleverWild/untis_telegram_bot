//! A lightweight lesson shape containing only fields used for message formatting.
//! This decouples message rendering from the full DB model and Untis model.

use chrono::NaiveTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLesson {
    // Core identity not required for rendering; lesson_id is omitted on purpose
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub lesson_code: String,
    pub subst_text: Option<String>,
    pub subjects: Vec<String>,
    pub teachers: Vec<String>,
    pub rooms: Vec<String>,
    pub classes: Vec<String>,
}

impl From<&db::models::Lesson> for MessageLesson {
    fn from(src: &db::models::Lesson) -> Self {
        Self {
            start_time: src.start_time,
            end_time: src.end_time,
            lesson_code: src.lesson_code.clone(),
            subst_text: src.subst_text.clone(),
            subjects: src.subjects.clone(),
            teachers: src.teachers.clone(),
            rooms: src.rooms.clone(),
            classes: src.classes.clone(),
        }
    }
}

impl From<&untis::Lesson> for MessageLesson {
    fn from(src: &untis::Lesson) -> Self {
        let subjects = src.subjects.iter().map(|i| i.name.clone()).collect();
        let teachers = src.teachers.iter().map(|i| i.name.clone()).collect();
        let rooms = src.rooms.iter().map(|i| i.name.clone()).collect();
        let classes = src.classes.iter().map(|i| i.name.clone()).collect();

        // Normalize lesson_code to a simple string identical to DB (e.g., "Regular")
        let lesson_code = serde_json::to_string(&src.code)
            .unwrap()
            .trim_matches('"')
            .to_string();

        Self {
            start_time: src.start_time.0,
            end_time: src.end_time.0,
            lesson_code,
            subst_text: src.subst_text.clone(),
            subjects,
            teachers,
            rooms,
            classes,
        }
    }
}
