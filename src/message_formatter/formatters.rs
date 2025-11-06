//! Specialized formatters for different message types

use crate::message_formatter::{
    LessonDiffBlock, MessageField, MessageHeader,
    fields::{FIELD_EXTRACTOR_CONFIG, LessonFieldExtractor},
};

use super::fields::{Field, FieldDiff, FieldVisibility};

/// Helper function to handle field visibility fallback
fn visibility_fallback(
    field: &LessonFieldExtractor,
    value: String,
    config: FieldVisibility,
) -> Option<MessageField> {
    if config == FieldVisibility::Always || !value.is_empty() {
        Some(MessageField::Normal(Field {
            name: field.name,
            value,
        }))
    } else {
        None
    }
}

/// Helper function to format lesson fields into MessageField variants.
/// If `from` is provided, compares with `to` for changes; otherwise, formats `to` as new.
pub fn format_lesson_fields(
    from: Option<&db::models::UnownedLesson>,
    to: &db::models::UnownedLesson,
) -> LessonDiffBlock {
    let mfs = FIELD_EXTRACTOR_CONFIG
        .iter()
        .filter_map(|field| {
            let to_value = field.extract(to);
            if let Some(from) = from
                && let Some(diff) = FieldDiff::new(
                    field.name,
                    field.extract(from),
                    to_value.clone(),
                    field.config,
                )
            {
                Some(MessageField::Changed(diff))
            } else {
                visibility_fallback(field, to_value, field.config)
            }
        })
        .collect();
    match from {
        Some(_) => LessonDiffBlock::new(MessageHeader::Changed, mfs),
        None => LessonDiffBlock::new(MessageHeader::Added, mfs),
    }
}

// /// Creates a Message for a lesson change
// pub fn format_change(
//     from: db::models::UnownedLesson,
//     to: db::models::UnownedLesson,
// ) -> LessonDiffBlock {
//     let field_diffs = format_lesson_fields(Some(&from), &to);
//     LessonDiffBlock::new(MessageHeader::Changed, field_diffs)
// }

// pub fn format_new(lesson: db::models::UnownedLesson) -> LessonDiffBlock {
//     let fields = format_lesson_fields(None, &lesson);
//     LessonDiffBlock::new(MessageHeader::Added, fields)
// }
