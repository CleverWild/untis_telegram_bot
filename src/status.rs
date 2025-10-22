use std::time::Duration;

use chrono::{FixedOffset, Utc};
use tokio::time::Instant;
use untis::{Date, Homework};

use crate::message::LabeledMessage;

const REPOSITORY_URL: &str = "https://github.com/CleverWild/untis_telegram_bot";

#[derive(Debug, Clone)]
pub struct StatusMessage {
    homeworks: Vec<Homework>,
    timestamp: Instant,
}

impl StatusMessage {
    pub fn new(homeworks: Vec<Homework>, uptime_since: Instant) -> Self {
        let sorted_hw = {
            let mut hw = homeworks
                .iter()
                .filter(|&h| !h.is_completed)
                .cloned()
                .collect::<Vec<_>>();
            hw.sort_by(|a, b| a.due_date.cmp(&b.due_date).then(a.date.cmp(&b.date)));
            hw
        };
        Self {
            homeworks: sorted_hw,
            timestamp: uptime_since,
        }
    }

    pub fn into_message(self) -> LabeledMessage {
        let mut msg = LabeledMessage::new();
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
