use std::fmt::Display;

use crate::{diff_impl::Diff, PROD};

pub fn format_message(diff: &Diff) -> String {
    match diff {
        Diff::Changed { from, to } => format_changed_diff(from, to),
        Diff::AddedNormal(new) => String::new(),
        Diff::AddedIrregular(new) => format_added_diff(new),
    }
}

fn format_changed_diff(from: &untis::Lesson, to: &untis::Lesson) -> String {
    let mut buffer = String::with_capacity(300);
    let mut has_diff = false;

    for (field_name, old_value, new_value, is_optional) in [
        (
            "Subject",
            from.subjects[0].name.clone(),
            to.subjects[0].name.clone(),
            false,
        ),
        (
            "Time",
            format!("{} - {}", from.start_time, from.end_time),
            format!("{} - {}", to.start_time, to.end_time),
            false,
        ),
        (
            "Teacher",
            from.teachers.first().cloned().unwrap_or_default().name,
            to.teachers.first().cloned().unwrap_or_default().name,
            false,
        ),
        (
            "Room",
            from.rooms[0].name.clone(),
            to.rooms[0].name.clone(),
            false,
        ),
        ("Status", from.code.to_string(), to.code.to_string(), false),
        (
            "Activity Type",
            from.activity_type.clone(),
            to.activity_type.clone(),
            true,
        ),
        (
            "Additional Info",
            from.subst_text.clone().unwrap_or_default(),
            to.subst_text.clone().unwrap_or_default(),
            true,
        ),
    ] {
        let result = format_maybe_diff(field_name, old_value, new_value, is_optional);
        if let FormatMaybeDiff::Diff(_) = result {
            has_diff = true;
        }
        buffer.push_str(&result.to_string());
    }

    if has_diff {
        buffer
    } else {
        format!("Not found any changes for lessons: <{from:?}> and <{to:?}>\n")
    }
}

fn format_added_diff(lesson: &untis::Lesson) -> String {
    format!(
        "New record for lesson: {}\nRoom: {}\nTime: {} - {}\nStatus: {:?}\n",
        lesson.subjects[0].name,
        lesson.rooms[0].name,
        lesson.start_time,
        lesson.end_time,
        lesson.code
    )
}

pub enum FormatMaybeDiff {
    Diff(String),
    NoDiff(String),
    Ignored(String),
}

impl Display for FormatMaybeDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormatMaybeDiff::Diff(diff) => write!(f, "{diff}"),
            FormatMaybeDiff::NoDiff(no_diff) => write!(f, "{no_diff}"),
            FormatMaybeDiff::Ignored(ignored) => {
                if PROD {
                    Ok(())
                } else {
                    write!(f, "<DEBUG: ignored {ignored}>")
                }
            }
        }
    }
}

fn format_maybe_diff(
    name: &str,
    from: String,
    to: String,
    show_only_changes: bool,
) -> FormatMaybeDiff {
    match (from == to, show_only_changes) {
        (true, true) => FormatMaybeDiff::Ignored(format!("{name}: {from}")),
        (true, false) => FormatMaybeDiff::NoDiff(format!("{name}: {from}\n")),
        (false, _) => FormatMaybeDiff::Diff(format!("{name}: {from} -> {to}\n")),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{diff_impl::Diff, message_formatter::format_message};

    #[test]
    fn test_format_message() {
        let prev = untis::Lesson {
            id: 2037263,
            date: untis::Date(NaiveDate::from_ymd_opt(2025, 5, 27).unwrap()),
            start_time: untis::Time(chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            end_time: untis::Time(chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap()),
            subjects: vec![untis::IdItem {
                id: 12,
                name: "Math".to_string(),
                orig_id: None,
                orig_name: None,
            }],
            rooms: vec![untis::IdItem {
                id: 34,
                name: "Room 101".to_string(),
                orig_id: None,
                orig_name: None,
            }],
            code: untis::LessonCode::Regular,
            lesson_type: untis::LessonType::Lesson,
            lsnumber: 1,
            lstext: "Math".to_string(),
            subst_text: None,
            classes: vec![],
            teachers: vec![untis::IdItem {
                id: 56,
                name: "John Doe".to_string(),
                orig_id: None,
                orig_name: None,
            }],
            statflags: "".to_string(),
            activity_type: "Unterricht".to_string(),
        };

        let mut new = prev.clone();
        new.rooms[0].id = 78;
        new.rooms[0].name = "Room 202".to_string();
        new.rooms[0].orig_id = Some(77);
        new.rooms[0].orig_name = Some("Room 101".to_string());
        // new.code = untis::LessonCode::Cancelled;

        let old = [prev];
        let new = [new];
        let diff = Diff::find(&old, &new);

        let message = format_message(&diff[0]);
        println!("{message}");
    }
}
