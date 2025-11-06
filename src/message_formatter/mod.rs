//! Core formatting functionality - the basic message dispatcher and public API
pub mod fields;
pub mod formatters;

use crate::{
    diff_impl::Diff,
    message::LabeledMessage,
    message_formatter::fields::{FIELD_SEPARATOR, Field, FieldDiff},
};

/// Primary entry point for formatting lesson difference messages
pub fn format_message(diff: Diff) -> LessonDiffBlock {
    let (from, to) = match diff {
        Diff::Changed { from, to } => (Some(&from.to_owned().into()), &to.to_owned().into()),
        Diff::Added(lesson) => (None, &lesson.to_owned().into()),
    };

    formatters::format_lesson_fields(from, to)
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
pub struct LessonDiffBlock {
    header: MessageHeader,
    fields: Vec<MessageField>,
}

impl LessonDiffBlock {
    pub fn new(with_type: MessageHeader, fields: Vec<MessageField>) -> Self {
        Self {
            header: with_type,
            fields,
        }
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
            if i != 0 {
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
                        msg.extend(formatted);
                    }
                }
            }
        }

        msg
    }
}

pub fn apply_debug_info<'a>(
    message: &'a mut LabeledMessage,
    _entry: &db::models::BotTask,
    diff: &Diff,
) -> &'a mut LabeledMessage {
    // Add debug information to the message
    match diff {
        Diff::Changed { from, to } => {
            message.push("Debug info: Changed lesson from ");
            message.push(format!("{:?}", from));
            message.push(" to ");
            message.push(format!("{:?}", to));
        }
        Diff::Added(lesson) => {
            message.push("Debug info: Added new lesson: ");
            message.push(format!("{:?}", lesson));
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

        let message = LessonDiffBlock::new(MessageHeader::Changed, fields);
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

        let message = LessonDiffBlock::new(MessageHeader::Added, fields);
        let labeled = message.into_labeled_message();
        let output = labeled.to_string();

        assert!(output.contains("New lesson"));
        assert!(output.contains("Math"));
    }
}
