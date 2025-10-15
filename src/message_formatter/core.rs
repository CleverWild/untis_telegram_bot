//! Core formatting functionality - the basic message dispatcher and public API

use super::formatters::{LessonChangeFormatter, NewLessonFormatter};
use crate::{
    diff_impl::Diff,
    message::LabeledMessage,
    message_formatter::fields::{FIELD_SEPARATOR, Field, FieldDiff},
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
pub fn format_message(diff: &Diff) -> LessonMessage {
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
    pub(super) fn dispatch(&self, diff: &Diff) -> LessonMessage {
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
}

#[derive(Debug, Clone)]
pub struct LessonMessage {
    header: MessageHeader,
    fields: Vec<MessageField>,
}

impl LessonMessage {
    pub fn new(with_type: MessageHeader, fields: Vec<MessageField>) -> Self {
        Self {
            header: with_type,
            fields,
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(MessageField::Normal(field));
    }

    /// Convert this LessonMessage into a LabeledMessage for unified message handling
    pub fn into_labeled_message(self) -> LabeledMessage {
        let mut msg = LabeledMessage::new();

        match self.header {
            MessageHeader::Changed => {
                msg.push("Changes in lesson:").nl();
            }
            MessageHeader::Added => {
                msg.push("New lesson:").nl();
            }
        }

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                msg.nl();
            }

            match field {
                MessageField::Normal(field) => {
                    msg.push(field.name);
                    msg.push(FIELD_SEPARATOR);
                    msg.push(&field.value);
                }
                MessageField::Changed(diff) => {
                    if let Some(formatted) = diff.format() {
                        msg.push(formatted);
                    }
                }
            }
        }

        msg
    }
}

pub fn apply_debug_info<'a>(
    message: &'a mut LabeledMessage,
    _entry: &WhitelistEntry,
    diff: &Diff,
) -> &'a mut LabeledMessage {
    // Add debug information to the message
    match diff {
        Diff::Changed { from, to } => {
            message.push("Debug info: Changed lesson from ");
            message.push(from.subjects.first().map_or("None", |s| &s.name));
            message.push(" to ");
            message.push(to.subjects.first().map_or("None", |s| &s.name));
        }
        Diff::Added(lesson) => {
            message.push("Debug info: Added new lesson for ");
            message.push(lesson.subjects.first().map_or("None", |s| &s.name));
        }
    }
    message
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
        ];

        let message = LessonMessage::new(MessageHeader::Changed, fields);
        let labeled = message.into_labeled_message();
        println!("{}", labeled);
    }

    #[test]
    fn test_labeled_message_conversion() {
        // Test that LessonMessage properly converts to LabeledMessage
        let fields = vec![MessageField::Normal(Field {
            name: "Subject",
            value: "Math".to_string(),
        })];

        let message = LessonMessage::new(MessageHeader::Added, fields);
        let labeled = message.into_labeled_message();
        let output = labeled.to_string();

        assert!(output.contains("New lesson"));
        assert!(output.contains("Math"));
    }
}
