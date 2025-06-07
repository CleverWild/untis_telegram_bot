//! Field system for extracting and formatting lesson data

/// Default buffer capacity for message formatting to avoid reallocations
pub const DEFAULT_BUFFER_CAPACITY: usize = 512;

pub const FIELD_SEPARATOR: &str = ": ";
pub const CHANGES_SEPARATOR: &str = " → ";

/// Type alias for field extraction functions
pub type FieldExtractor = fn(&untis::Lesson) -> String;

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
    /// Hide when not empty
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
    pub fn extract(&self, lesson: &untis::Lesson) -> String {
        (self.extractor)(lesson)
    }
}

/// Registry of all available lesson fields
pub struct FieldRegistry;

impl FieldRegistry {
    /// Returns all standard lesson fields
    pub fn standard_fields() -> Vec<LessonField> {
        vec![
            LessonField::required("Subject", |l| {
                l.subjects
                    .first()
                    .map_or(String::new(), |subject| subject.name.clone())
            }),
            LessonField::required("Time", |l| format!("{} - {}", l.start_time, l.end_time)),
            LessonField::required("Teacher", |l| {
                l.teachers
                    .iter()
                    .map(|teacher| teacher.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            LessonField::required("Room", |l| {
                l.rooms
                    .first()
                    .map_or(String::new(), |room| room.name.clone())
            }),
            LessonField::required("Status", |l| l.code.to_string()),
            LessonField::only_changed("Activity Type", |l| l.activity_type.clone()),
            LessonField::only_changed("Additional Info", |l| {
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

    /// Formats the field difference for display
    pub fn format(&self) -> Option<String> {
        if !self.has_changes() {
            if self.visibility != FieldVisibility::Changed && !self.from.is_empty() {
                Some(format!("{}{FIELD_SEPARATOR}{}", self.name, self.from))
            } else {
                None
            }
        } else {
            match (self.from.is_empty(), self.to.is_empty()) {
                (true, false) => Some(format!("Added {}{FIELD_SEPARATOR}{}", self.name, self.to)),
                (false, false) => Some(format!(
                    "{}{FIELD_SEPARATOR}{}{CHANGES_SEPARATOR}{}",
                    self.name, self.from, self.to
                )),
                (false, true) => Some(format!(
                    "{}{FIELD_SEPARATOR}{}",
                    self.name,
                    make_strikethrough(&self.from)
                )),
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

    pub fn field_from(&self) -> Field {
        Field {
            name: self.name,
            value: self.from.clone(),
        }
    }

    pub fn field_to(&self) -> Field {
        Field {
            name: self.name,
            value: self.to.clone(),
        }
    }
}

fn make_strikethrough(text: &str) -> String {
    format!("~~{text}~~")
}
