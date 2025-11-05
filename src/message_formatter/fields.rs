//! Field system for extracting and formatting lesson data

use super::message_lesson::MessageLesson;
use crate::message::LabeledMessage;

/// Default buffer capacity for message formatting to avoid reallocations
pub const DEFAULT_BUFFER_CAPACITY: usize = 512;

pub const FIELD_SEPARATOR: &str = ": ";
pub const CHANGES_SEPARATOR: &str = " → ";

/// Type alias for field extraction functions
pub type FieldExtractor = fn(&MessageLesson) -> String;

/// Represents a lesson field that can be extracted and formatted
#[derive(Clone)]
pub struct LessonField {
    /// Human-readable name of the field
    pub name: &'static str,
    /// Function to extract the field value from a lesson
    pub extractor: FieldExtractor,
    /// Configuration for this field
    pub config: FieldVisibility,
}

/// Configuration options for lesson fields
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldVisibility {
    /// Always show this field, even if empty
    Always,
    /// Show when not empty
    NonEmpty,
    /// Show only when changed
    Changed,
}

impl LessonField {
    /// Creates a new required field
    pub const fn required(name: &'static str, extractor: FieldExtractor) -> Self {
        Self {
            name,
            extractor,
            config: FieldVisibility::Always,
        }
    }

    /// Creates a new optional field that always shows
    pub const fn non_empty(name: &'static str, extractor: FieldExtractor) -> Self {
        Self {
            name,
            extractor,
            config: FieldVisibility::NonEmpty,
        }
    }

    /// Creates a new optional field that only shows when changed
    pub const fn only_changed(name: &'static str, extractor: FieldExtractor) -> Self {
        Self {
            name,
            extractor,
            config: FieldVisibility::Changed,
        }
    }

    /// Extracts the field value from a lesson
    pub fn extract(&self, lesson: &MessageLesson) -> String {
        (self.extractor)(lesson)
    }
}

/// Registry of all available lesson fields
pub struct FieldRegistry;

impl FieldRegistry {
    /// Returns all standard lesson fields
    pub const fn standard_fields() -> [LessonField; 6] {
        [
            LessonField::required("Subject", |l| {
                l.subjects
                    .first()
                    .map_or(String::new(), |subject| subject.clone())
            }),
            LessonField::required("Time", |l| format!("{} - {}", l.start_time, l.end_time)),
            LessonField::required("Teacher", |l| l.teachers.to_vec().join(", ")),
            LessonField::required("Room", |l| l.rooms.to_vec().join(", ")),
            LessonField::required("Status", |l| l.lesson_code.clone()),
            LessonField::required("Additional Info", |l| {
                l.subst_text.clone().unwrap_or_default()
            }),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Name of the field
    pub name: &'static str,
    pub value: String,
}

/// Represents a difference between two field values
///
/// Invariant: `from` and `to` are not equal
#[derive(Debug, Clone)]
pub struct FieldDiff {
    pub name: &'static str,
    pub from: String,
    pub to: String,
    pub visibility: FieldVisibility,
}

impl FieldDiff {
    /// Creates a new field difference
    pub fn new(
        name: &'static str,
        from_value: String,
        to_value: String,
        visibility: FieldVisibility,
    ) -> Option<Self> {
        if from_value != to_value {
            Some(Self {
                name,
                from: from_value,
                to: to_value,
                visibility,
            })
        } else {
            None
        }
    }

    /// Checks if this field has actual changes
    pub fn has_changes(&self) -> bool {
        self.from != self.to
    }

    /// Formats the field difference for display with MarkdownV2 escaping
    pub fn format(&self) -> Option<LabeledMessage> {
        let mut msg = LabeledMessage::new();
        if !self.has_changes() {
            if self.visibility != FieldVisibility::Changed && !self.from.is_empty() {
                msg.push(self.name).push(FIELD_SEPARATOR).push(&self.from);
                Some(msg)
            } else {
                None
            }
        } else {
            match (self.from.is_empty(), self.to.is_empty()) {
                (true, false) => {
                    msg.push("Added ")
                        .push(self.name)
                        .push(FIELD_SEPARATOR)
                        .push_code_inline(&self.to);
                    Some(msg)
                }
                (false, false) => {
                    msg.push(self.name)
                        .push(FIELD_SEPARATOR)
                        .push_code_inline(&self.from)
                        .push(CHANGES_SEPARATOR)
                        .push_code_inline(&self.to);
                    Some(msg)
                }
                (false, true) => {
                    msg.push(self.name)
                        .push(FIELD_SEPARATOR)
                        .enter_strikethrough(|msg| msg.push_code_inline(&self.from));
                    Some(msg)
                }
                _ => {
                    tracing::warn!(
                        "Unexpected field diff state: from_value='{}', to_value='{}'",
                        self.from,
                        self.to
                    );
                    None
                }
            }
        }
    }

    // pub fn field_from(&self) -> Field {
    //     Field {
    //         name: self.name,
    //         value: self.from.clone(),
    //     }
    // }

    // pub fn field_to(&self) -> Field {
    //     Field {
    //         name: self.name,
    //         value: self.to.clone(),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_no_change_visible() {
        let diff = FieldDiff {
            name: "Room",
            from: "101".into(),
            to: "101".into(),
            visibility: FieldVisibility::Always,
        };
        let formatted = diff.format();
        assert_eq!(
            formatted.map(|s| s.to_string()),
            Some("Room: 101".to_string())
        );
    }

    #[test]
    fn format_no_change_hidden_when_changed_visibility() {
        let diff = FieldDiff {
            name: "Status",
            from: "Regular".into(),
            to: "Regular".into(),
            visibility: FieldVisibility::Changed,
        };
        assert!(diff.format().is_none());
    }

    #[test]
    fn format_added() {
        let diff = FieldDiff {
            name: "Activity Type",
            from: String::new(),
            to: "Exam".into(),
            visibility: FieldVisibility::Changed,
        };
        let formatted = diff.format().map(|msg| msg.to_string());
        assert_eq!(formatted.as_deref(), Some("Added Activity Type: `Exam`"));
    }

    #[test]
    fn format_changed() {
        let diff = FieldDiff {
            name: "Room",
            from: "101".into(),
            to: "202".into(),
            visibility: FieldVisibility::Changed,
        };
        let formatted = diff.format().map(|msg| msg.to_string());
        assert_eq!(
            formatted.as_deref(),
            Some(format!("Room: `101`{CHANGES_SEPARATOR}`202`").as_str())
        );
    }

    #[test]
    fn format_removed() {
        let diff = FieldDiff {
            name: "Teacher",
            from: "Smith".into(),
            to: String::new(),
            visibility: FieldVisibility::Changed,
        };
        // MarkdownV2 uses single tilde for strikethrough
        let formatted = diff.format().map(|msg| msg.to_string());
        assert_eq!(formatted.as_deref(), Some("Teacher: ~`Smith`~"));
    }

    #[test]
    fn format_both_empty() {
        let diff = FieldDiff {
            name: "Additional Info",
            from: String::new(),
            to: String::new(),
            visibility: FieldVisibility::Always,
        };
        assert!(diff.format().is_none());
    }
}
