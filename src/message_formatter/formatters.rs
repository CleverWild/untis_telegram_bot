//! Specialized formatters for different message types

use crate::message_formatter::core::{Message, MessageField, MessageHeader};

use super::fields::{FieldDiff, FieldRegistry, LessonField};

/// Formats lesson changes by comparing before and after states
pub struct LessonChangeFormatter {
    pub fields: Vec<LessonField>,
}

impl LessonChangeFormatter {
    /// Creates a new lesson change formatter
    pub fn new() -> Self {
        Self {
            fields: FieldRegistry::standard_fields(),
        }
    }

    /// Creates a Message for a lesson change
    pub fn format_change(&self, from: &untis::Lesson, to: &untis::Lesson) -> Message {
        let field_diffs = self
            .fields
            .iter()
            .filter_map(|field| {
                FieldDiff::new(
                    field.name,
                    field.extract(from),
                    field.extract(to),
                    field.config,
                )
                .map(MessageField::Changed)
            })
            .collect::<Vec<_>>();

        Message::new(MessageHeader::Changed, field_diffs)
    }
}

impl Default for LessonChangeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats new lesson notifications
#[derive(Default)]
pub struct NewLessonFormatter;

impl NewLessonFormatter {
    /// Creates a Message for a new lesson notification
    pub fn format_new(&self, lesson: &untis::Lesson) -> Message {
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

        Message::new(MessageHeader::Added, fields)
    }
}
