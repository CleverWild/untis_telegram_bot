use std::fmt::Display;

use teloxide::utils::markdown::escape;

use crate::IS_PROD;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LabeledString {
    Normal(String),
    Debug(String),
}

impl Display for LabeledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabeledString::Normal(text) => write!(f, "{text}"),
            LabeledString::Debug(text) => {
                write!(
                    f,
                    "{}```\n{text}\n```",
                    teloxide::utils::markdown::bold(&escape("<---DEBUG--->"))
                )
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LabeledMessage(Vec<LabeledString>);
#[allow(deprecated)]
impl LabeledMessage {
    pub fn new() -> Self {
        Self::default()
    }

    #[deprecated(note = "Try to avoid using this method")]
    pub fn push_raw(&mut self, text: impl ToString) -> &mut Self {
        self.0.push(LabeledString::Normal(text.to_string()));
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

    pub fn extend(&mut self, other: LabeledMessage) -> &mut Self {
        self.0.extend(other.0);
        self
    }

    pub fn debug_ln<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self) -> &mut Self,
    {
        let mut tmp = Self::new();
        f(&mut tmp);

        for entry in &mut tmp.0 {
            if let LabeledString::Normal(s) = entry {
                let content = std::mem::take(s);
                *entry = LabeledString::Debug(content);
            }
        }

        self.nl().0.extend(tmp.0);
    }

    pub fn filter_normal(&self) -> Self {
        Self(
            self.0
                .iter()
                .filter(|s| matches!(**s, LabeledString::Normal(_)))
                .cloned()
                .collect(),
        )
    }
}

impl Display for LabeledMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display: String = if IS_PROD {
            self.0
                .iter()
                .filter(|s| matches!(**s, LabeledString::Normal(_)))
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
    use crate::{DEBUG_TELEGRAM_CHAT, utils::send_message};

    use super::*;
    use teloxide::Bot;

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

            // Sample 4: Only debug (should show nothing in normal view)
            let mut m4 = LabeledMessage::new();
            m4.debug_ln(|msg| msg.push("Debug: orphan diff entry (sanity check)"));
            v.push(m4);

            v
        };

        for (i, msg_part) in samples.iter().enumerate() {
            let mut msg = LabeledMessage::new();
            msg.push("Sample ");
            msg.push_code_inline(format!("#{i}")).nl();
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
