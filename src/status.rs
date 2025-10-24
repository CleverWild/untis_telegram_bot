use std::time::Duration;

use chrono::{FixedOffset, Local, NaiveTime, Utc};
use tokio::time::Instant;
use untis::{Homework, Lesson};

use crate::message::LabeledMessage;

const REPOSITORY_URL: &str = "https://github.com/CleverWild/untis_telegram_bot";

#[derive(Debug, Clone)]
pub struct StatusMessage {
    homeworks: Vec<Homework>,
    current_or_next_lesson: Option<NearestLesson>,
    timestamp: Instant,
}

#[derive(Debug, Clone)]
enum NearestLesson {
    Current(Lesson),
    Next(Lesson),
}

impl StatusMessage {
    pub fn new(homeworks: Vec<Homework>, timetable: &[Lesson], uptime_since: Instant) -> Self {
        let sorted_hw = {
            let mut hw = homeworks
                .iter()
                .filter(|&h| !h.is_completed)
                .cloned()
                .collect::<Vec<_>>();
            hw.sort_by(|a, b| a.due_date.cmp(&b.due_date).then(a.date.cmp(&b.date)));
            hw
        };

        // Find nearest lesson
        let current_or_next_lesson = find_nearest_lesson(timetable);

        Self {
            homeworks: sorted_hw,
            current_or_next_lesson,
            timestamp: uptime_since,
        }
    }

    pub fn into_message(self) -> LabeledMessage {
        let mut msg = LabeledMessage::new();

        // Show nearest lesson
        if let Some(lesson_info) = &self.current_or_next_lesson {
            match lesson_info {
                NearestLesson::Current(lesson) => {
                    msg.push_bold("Current Lesson:").nl();
                    format_lesson(&mut msg, lesson);
                }
                NearestLesson::Next(lesson) => {
                    msg.push_bold("Next Lesson:").nl();
                    format_lesson(&mut msg, lesson);
                }
            }
            msg.nl();
        }

        msg.push_bold("Homework list:").nl();

        if self.homeworks.is_empty() {
            msg.push_code_inline("Empty :)").nl();
        } else {
            for hw in self.homeworks {
                msg.push("• Lesson: ")
                    .push_code_inline(hw.lesson.subject)
                    .push(", Teacher: ")
                    .push_code_inline(hw.teacher.name)
                    .nl();

                let hw_date_format = "%a %d/%m/%y";
                msg.push("  Created at:  ")
                    .push_code_inline(hw.date.format(hw_date_format).to_string())
                    .nl();
                msg.push("  Deadline:     ")
                    .push_code_inline(hw.due_date.format(hw_date_format).to_string())
                    .nl();

                msg.push("  Task:  ").push_code_inline(hw.text).nl().nl();
            }
        }

        // Added uptime calculation (days:hours:minutes)
        let elapsed = self.timestamp.elapsed();
        msg.push("Uptime:  ")
            .push_code_inline(into_uptime(elapsed))
            .nl();

        let offset = FixedOffset::east_opt(2 * 3600).expect("valid offset"); // GMT+2 fixed (no DST)
        msg.push("Last refresh:  ")
            .push_code_inline(
                Utc::now()
                    .with_timezone(&offset)
                    .format("%H:%M:%S %a %d/%m/%y")
                    .to_string(),
            )
            .nl();

        msg.enter_spoiler(|msg| {
            msg.push_link(
                "(He keeps me in this basement full of care)",
                REPOSITORY_URL,
            )
            .nl()
        });
        msg
    }
}

fn plural(n: u64, one: &str, many: &str) -> String {
    if n == 1 {
        one.to_string()
    } else {
        many.to_string()
    }
}

fn into_uptime(d: Duration) -> String {
    let total_minutes = d.as_secs() / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let minutes = total_minutes % 60;

    format!(
        "{d} {}, {h} {}, {m} {}",
        plural(days, "day", "days"),
        plural(hours, "hour", "hours"),
        plural(minutes, "minute", "minutes"),
        d = days,
        h = hours,
        m = minutes
    )
}

/// Find the current lesson (if ongoing) or the next upcoming lesson
fn find_nearest_lesson(timetable: &[Lesson]) -> Option<NearestLesson> {
    use chrono::Timelike;

    let now = Local::now();
    let today = now.date_naive();
    let current_time = NaiveTime::from_hms_opt(now.hour(), now.minute(), 0)?;

    // First, check if there's a current lesson (today, ongoing)
    for lesson in timetable {
        if lesson.date.0 == today
            && lesson.start_time.0 <= current_time
            && current_time < lesson.end_time.0
        {
            return Some(NearestLesson::Current(lesson.clone()));
        }
    }

    // If no current lesson, find the next one
    let mut future_lessons: Vec<_> = timetable
        .iter()
        .filter(|lesson| {
            lesson.date.0 > today || (lesson.date.0 == today && lesson.start_time.0 > current_time)
        })
        .collect();

    future_lessons.sort_by_key(|lesson| (lesson.date, lesson.start_time));

    future_lessons
        .first()
        .map(|&lesson| NearestLesson::Next(lesson.clone()))
}

/// Format a lesson for display in status message
fn format_lesson(msg: &mut LabeledMessage, lesson: &Lesson) {
    // Subject
    let subjects: Vec<_> = lesson.subjects.iter().map(|s| s.name.as_str()).collect();
    msg.push("  Subject: ")
        .push_code_inline(subjects.join(", "));

    // Teacher
    let teachers: Vec<_> = lesson.teachers.iter().map(|t| t.name.as_str()).collect();
    if !teachers.is_empty() {
        msg.push(", Teacher: ")
            .push_code_inline(teachers.join(", "));
    }

    // Room
    let rooms: Vec<_> = lesson.rooms.iter().map(|r| r.name.as_str()).collect();
    if !rooms.is_empty() {
        msg.push(", Room: ").push_code_inline(rooms.join(", "));
    }
    msg.nl();

    // Time
    msg.push("  Time: ").push_code_inline(format!(
        "{} - {}",
        lesson.start_time.0.format("%H:%M"),
        lesson.end_time.0.format("%H:%M")
    ));
    // Date (if not today)
    if lesson.date.0 != Local::now().date_naive() {
        msg.push("   ")
            .push_code_inline(lesson.date.0.format("%a %d/%m/%y").to_string());
    }
    msg.nl();
}
