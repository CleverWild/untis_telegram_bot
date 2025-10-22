use std::fmt::Display;

use teloxide::utils::markdown::escape;

use crate::IS_PROD;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LabeledMessageInner {
    Normal(String),
    Debug(LabeledMessage),
    Strikethrough(LabeledMessage),
}

impl Display for LabeledMessageInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(text) => write!(f, "{text}"),
            Self::Debug(msg) => {
                write!(
                    f,
                    "{}```\n{}\n```",
                    teloxide::utils::markdown::bold(&escape("<---DEBUG--->")),
                    msg
                )
            }
            Self::Strikethrough(msg) => write!(f, "~{}~", msg),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabeledMessage(Vec<LabeledMessageInner>);
#[allow(deprecated)]
impl LabeledMessage {
    pub fn new() -> Self {
        Self::default()
    }

    #[deprecated(note = "Try to avoid using this method")]
    pub fn push_raw(&mut self, text: impl ToString) -> &mut Self {
        self.0.push(LabeledMessageInner::Normal(text.to_string()));
        self
    }

    pub fn nl(&mut self) -> &mut Self {
        self.push_raw('\n')
    }

    pub fn push(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.push_raw(escape(text.as_ref()))
    }

    pub fn push_bold(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.push_raw(teloxide::utils::markdown::bold(&escape(text.as_ref())))
    }

    pub fn push_italic(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.push_raw(teloxide::utils::markdown::italic(&escape(text.as_ref())))
    }

    pub fn push_code(&mut self, text: impl AsRef<str>, lang: Option<&'static str>) -> &mut Self {
        let code = match lang {
            Some(l) => teloxide::utils::markdown::code_block_with_lang(text.as_ref(), l),
            None => teloxide::utils::markdown::code_block(text.as_ref()),
        };
        self.push_raw(code)
    }

    pub fn push_code_inline(&mut self, text: impl AsRef<str>) -> &mut Self {
        self.push_raw(teloxide::utils::markdown::code_inline(text.as_ref()))
    }

    pub fn enter_strikethrough<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self) -> &mut Self,
    {
        let mut tmp = Self::new();
        f(&mut tmp);
        self.0.push(LabeledMessageInner::Strikethrough(tmp));
        self
    }

    pub fn extend(&mut self, other: Self) -> &mut Self {
        self.0.extend(other.0);
        self
    }

    pub fn debug_ln<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self) -> &mut Self,
    {
        let mut tmp = Self::new();
        f(&mut tmp);

        self.nl().0.push(LabeledMessageInner::Debug(tmp));
    }

    pub fn filter_normal(&self) -> Self {
        Self(
            self.0
                .iter()
                .filter_map(|s| match s {
                    LabeledMessageInner::Normal(_) => Some(s.clone()),
                    LabeledMessageInner::Debug(_) => None,
                    LabeledMessageInner::Strikethrough(msg) => {
                        Some(LabeledMessageInner::Strikethrough(msg.filter_normal()))
                    }
                })
                .collect(),
        )
    }
}

impl Display for LabeledMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display: String = if IS_PROD {
            self.filter_normal()
                .0
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            self.0.iter().map(|s| s.to_string()).collect()
        };

        write!(f, "{display}")
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DEBUG_TELEGRAM_CHAT, message_formatter::formatters::LessonChangeFormatter,
        utils::send_message,
    };

    use super::*;
    use teloxide::Bot;
    use untis::{IdItem, Lesson, LessonCode, LessonType};

    // Helper function to create test IdItem
    fn test_id_item(id: isize, name: &str) -> IdItem {
        IdItem {
            id,
            name: name.to_string(),
            orig_id: None,
            orig_name: None,
        }
    }

    // Helper function to create a test lesson
    fn test_lesson(
        id: usize,
        date_str: &str,
        start_hour: u32,
        end_hour: u32,
        subject: &str,
        teacher: &str,
        room: &str,
    ) -> Lesson {
        use chrono::NaiveDate;
        use untis::{Date, Time};

        let date_parts: Vec<&str> = date_str.split('-').collect();
        let year = date_parts[0].parse().unwrap();
        let month = date_parts[1].parse().unwrap();
        let day = date_parts[2].parse().unwrap();

        Lesson {
            id,
            date: Date(NaiveDate::from_ymd_opt(year, month, day).unwrap()),
            start_time: Time(chrono::NaiveTime::from_hms_opt(start_hour, 0, 0).unwrap()),
            end_time: Time(chrono::NaiveTime::from_hms_opt(end_hour, 0, 0).unwrap()),
            lesson_type: LessonType::Lesson,
            code: LessonCode::Regular,
            lsnumber: 1,
            lstext: String::new(),
            subst_text: None,
            classes: vec![test_id_item(1, "VABO1")],
            subjects: vec![test_id_item(10, subject)],
            rooms: vec![test_id_item(100, room)],
            teachers: vec![test_id_item(50, teacher)],
            statflags: String::new(),
            activity_type: "Unterricht".to_string(),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn send_test_messages() {
        let token = {
            let config = config::Config::builder()
                .add_source(config::File::with_name("Secrets.toml"))
                .build()
                .expect("Failed to load Secrets.toml");
            config
                .get::<String>("bot_token")
                .expect("Set TELOXIDE_TOKEN or TELEGRAM_BOT_TOKEN to run this test")
        };

        let bot = Bot::new(token);

        // Prepare sample messages with different combinations of normal & debug parts.
        let samples: Vec<LabeledMessage> = {
            let mut v = Vec::new();

            // Sample 1: Only normal text
            let mut m1 = LabeledMessage::new();
            m1.push("Changes in timetable:").nl();
            m1.push("Date: 2025-09-16 Tuesday").nl();
            m1.push("Subject: Math (Room 101)").nl();
            m1.push_code("123123", None).nl();
            v.push(m1);

            // Sample 2: Normal + debug
            let mut m2 = LabeledMessage::new();
            m2.push("Changes in timetable:").nl();
            m2.push("Room change: 101 → 202").nl();
            m2.debug_ln(|msg| msg.push("Debug: lesson_id=12345 original_room=101 new_room=202"));
            v.push(m2);

            // Sample 3: Multiple dates separated
            let mut m3 = LabeledMessage::new();
            m3.nl().push("Date: 2025-09-16 Tuesday").nl();
            m3.push("Physics → Chemistry (Lab)").nl();
            m3.debug_ln(|msg| msg.push("Debug: diff_type=SubjectSwap id=777"));
            m3.push("----------------").nl().nl();
            m3.push("Date: 2025-09-17 Wednesday").nl();
            m3.push("Added lesson: Biology Extra Session").nl();
            m3.debug_ln(|msg| msg.push("Debug: new_lesson_id=888 kind=Added"));
            v.push(m3);

            // Sample 4: Changed lesson (from/to comparison)
            let from_lesson =
                test_lesson(12345, "2025-09-18", 10, 11, "Mathematics", "Smith", "101");

            let to_lesson = test_lesson(
                12345,
                "2025-09-18",
                10,
                11,
                "Mathematics",
                "Johnson", // Teacher changed
                "202",     // Room changed
            );

            let m4 = LessonChangeFormatter
                .format_change(&from_lesson, &to_lesson)
                .into_labeled_message();
            v.push(m4);

            v
        };

        for (i, msg_part) in samples.iter().enumerate() {
            let mut msg = LabeledMessage::new();
            msg.push("Sample ");
            msg.push_code_inline(format!("#{}", i + 1)).nl();
            msg.push_bold("Normal view:").nl();
            msg.extend(msg_part.filter_normal()).nl().nl();
            msg.push_bold("Full view (with debug):").nl();
            msg.extend(msg_part.to_owned());

            println!("===== Sending test message {msg} =====");
            if let Err(e) = send_message(&bot, DEBUG_TELEGRAM_CHAT, msg.to_string()).await {
                panic!("Failed to send test message #{i}: {e:?}");
            }
            // Small delay to avoid hitting flood limits.
            // tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
