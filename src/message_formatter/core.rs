//! Core formatting functionality - the basic message dispatcher and public API

use std::fmt::Display;

use super::formatters::{LessonChangeFormatter, NewLessonFormatter};
use crate::{
    diff_impl::Diff,
    message_formatter::fields::{DEFAULT_BUFFER_CAPACITY, FIELD_SEPARATOR, Field, FieldDiff},
    work::WhitelistEntry,
};

/// Primary entry point for formatting lesson difference messages
///
/// # Arguments
///
/// * `diff` - The lesson difference to format
///
/// # Returns
///
/// A formatted message ready for display in Telegram
pub fn format_message(diff: &Diff) -> Message {
    MessageDispatcher::default().dispatch(diff)
}

/// Dispatches different types of lesson differences to appropriate formatters
#[derive(Default)]
pub(super) struct MessageDispatcher {
    lesson_change_formatter: LessonChangeFormatter,
    new_lesson_formatter: NewLessonFormatter,
}

impl MessageDispatcher {
    /// Dispatches the diff to the appropriate formatter
    pub(super) fn dispatch(&self, diff: &Diff) -> Message {
        match diff {
            Diff::Changed { from, to } => self.lesson_change_formatter.format_change(from, to),
            Diff::Added(lesson) => self.new_lesson_formatter.format_new(lesson),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageHeader {
    Added,
    Changed,
}

#[derive(Debug, Clone)]
pub enum MessageField {
    Normal(Field),
    Changed(FieldDiff),
    Raw(String),
}

#[derive(Debug, Clone)]
pub struct Message {
    header: MessageHeader,
    fields: Vec<MessageField>,
}

impl Message {
    pub fn new(with_type: MessageHeader, fields: Vec<MessageField>) -> Self {
        Self {
            header: with_type,
            fields,
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(MessageField::Normal(field));
    }

    pub fn pushln(&mut self, line: String) {
        self.fields.push(MessageField::Raw(line));
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = String::with_capacity(DEFAULT_BUFFER_CAPACITY);

        match self.header {
            MessageHeader::Changed => {
                buffer.push_str("Changes in lesson:\n");
            }
            MessageHeader::Added => {
                buffer.push_str("New lesson:\n");
            }
        }

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                buffer.push('\n');
            }

            match field {
                MessageField::Normal(field) => {
                    buffer.push_str(field.name);
                    buffer.push_str(FIELD_SEPARATOR);
                    buffer.push_str(&field.value);
                }
                MessageField::Changed(diff) => {
                    if let Some(formatted) = diff.format() {
                        buffer.push_str(&formatted);
                    }
                }
                MessageField::Raw(raw) => {
                    buffer.push_str(raw);
                }
            }
        }

        writeln!(f, "{buffer}")
    }
}

pub fn apply_debug_info(message: &mut Message, _entry: &WhitelistEntry, diff: &Diff) {
    // Add debug information to the message
    let debug_info = match diff {
        Diff::Changed { from, to } => format!(
            "Debug info: Changed lesson from {} to {}",
            from.subjects.first().map_or("None", |s| &s.name),
            to.subjects.first().map_or("None", |s| &s.name)
        ),
        Diff::Added(lesson) => format!(
            "Debug info: Added new lesson for {}",
            lesson.subjects.first().map_or("None", |s| &s.name)
        ),
    };

    message.pushln(debug_info);
}

#[cfg(test)]
mod tests {
    use crate::message_formatter::fields::FieldVisibility;

    use super::*;

    #[test]
    fn test_output() {
        let fields = vec![
            MessageField::Normal(Field {
                name: "Subject",
                value: "Math".to_string(),
            }),
            MessageField::Normal(Field {
                name: "Room",
                value: "101".to_string(),
            }),
            MessageField::Changed(FieldDiff {
                name: "Time",
                from: "08:00 - 09:30".to_string(),
                to: "09:00 - 10:30".to_string(),
                visibility: FieldVisibility::Always,
            }),
            MessageField::Raw("`This is a raw message`".to_string()),
        ];

        let message = Message::new(MessageHeader::Changed, fields);
        println!("{message}");
    }
}
