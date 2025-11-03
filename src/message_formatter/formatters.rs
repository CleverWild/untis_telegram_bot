//! Specialized formatters for different message types

use crate::message_formatter::core::{LessonMessage, MessageField, MessageHeader};

use super::fields::{FieldDiff, FieldRegistry};

/// Formats lesson changes by comparing before and after states
#[derive(Default)]
pub struct LessonChangeFormatter;

impl LessonChangeFormatter {
    /// Creates a Message for a lesson change
    pub fn format_change(&self, from: &db::models::Lesson, to: &untis::Lesson) -> LessonMessage {
        // Convert Untis::Lesson into a synthetic db::models::Lesson-like shape locally
        // to reuse the same field extractor logic without global coupling.
        let to_entry: db::models::Lesson = lesson_entry_from_untis(to);

        let field_diffs = FieldRegistry::standard_fields()
            .iter()
            .filter_map(|field| {
                let from_value = field.extract(from);
                let to_value = field.extract(&to_entry);

                if let Some(diff) =
                    FieldDiff::new(field.name, from_value.clone(), to_value, field.config)
                {
                    // Field has changes, show as changed
                    Some(MessageField::Changed(diff))
                } else if field.config == super::fields::FieldVisibility::Always
                    && !from_value.is_empty()
                {
                    // Field hasn't changed but is marked as required (Always) and has a value
                    Some(MessageField::Normal(super::fields::Field {
                        name: field.name,
                        value: from_value,
                    }))
                } else {
                    // Field hasn't changed and is not required, skip it
                    None
                }
            })
            .collect::<Vec<_>>();

        LessonMessage::new(MessageHeader::Changed, field_diffs)
    }
}

/// Formats new lesson notifications
#[derive(Default)]
pub struct NewLessonFormatter;

impl NewLessonFormatter {
    /// Creates a Message for a new lesson notification
    /// Note: Field values should NOT be pre-escaped as they will be escaped during Display formatting
    pub fn format_new(&self, lesson: &untis::Lesson) -> LessonMessage {
        use super::fields::Field;

        let fields = vec![
            Field {
                name: "Subject",
                value: lesson
                    .subjects
                    .first()
                    .map_or("Unknown".to_string(), |s| s.name.clone()),
            },
            Field {
                name: "Room",
                value: lesson
                    .rooms
                    .first()
                    .map_or("Unknown".to_string(), |r| r.name.clone()),
            },
            Field {
                name: "Time",
                value: format!("{} - {}", lesson.start_time, lesson.end_time),
            },
            Field {
                name: "Status",
                value: lesson.code.to_string(),
            },
        ]
        .into_iter()
        .map(MessageField::Normal)
        .collect::<Vec<_>>();

        LessonMessage::new(MessageHeader::Added, fields)
    }
}

/// Local helper: convert Untis::Lesson to a db::models::Lesson-compatible struct
/// for the purpose of field extraction/formatting only.
fn lesson_entry_from_untis(lesson: &untis::Lesson) -> db::models::Lesson {
    use db::Uuid;

    let subjects = lesson.subjects.iter().map(|i| i.name.clone()).collect();

    let teachers = lesson.teachers.iter().map(|i| i.name.clone()).collect();

    let rooms = lesson.rooms.iter().map(|i| i.name.clone()).collect();

    let classes = lesson.classes.iter().map(|i| i.name.clone()).collect();

    let lesson_type = Some(
        match lesson.lesson_type {
            untis::LessonType::Lesson => "Unterricht",
            untis::LessonType::OfficeHour => "oh",
            untis::LessonType::Standby => "sb",
            untis::LessonType::BreakSupervision => "bs",
            untis::LessonType::Exam => "ex",
        }
        .to_string(),
    );

    db::models::Lesson {
        subjects,
        teachers,
        rooms,
        classes,
        id: Uuid::nil(),
        lesson_id: lesson.id as i64,
        date: lesson.date.0,
        end_time: lesson.end_time.0,
        lesson_code: lesson.code.to_string(),
        lesson_type,
        start_time: lesson.start_time.0,
        subst_text: lesson.subst_text.clone(),
        bot_state: None,
    }
}
