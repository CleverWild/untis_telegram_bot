use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use teloxide::{
    Bot,
    payloads::{EditMessageTextSetters as _, SendMessageSetters as _},
    prelude::{Message, Request as _, Requester as _},
    sugar::request::RequestLinkPreviewExt,
    types::MessageId,
};
use tokio::time::Instant;

use crate::{diff_impl::Diff, work::Chat};

pub fn next_friday(from: NaiveDate) -> NaiveDate {
    let days_to_next_friday = (11 - from.weekday().num_days_from_monday()) % 7;
    from + chrono::Duration::days(days_to_next_friday as i64)
}

pub fn sort_diffs(diffs: &mut Vec<Diff>) {
    diffs.sort_by(|l, r| {
        l.date()
            .cmp(&r.date())
            .then_with(|| l.start_time().cmp(&r.start_time()))
            .then_with(|| l.end_time().cmp(&r.end_time()))
            .then_with(|| l.code().cmp(r.code()))
    })
}

pub async fn send_message(bot: &Bot, chat: Chat, text: String) -> Result<Message, eyre::Report> {
    tracing::info!(
        "Sending to chat `{}` with topic `{:?}` message:\n{}",
        chat.id,
        chat.thread_id,
        text
    );

    let mut req = bot
        .send_message(chat.id, text)
        .disable_link_preview(true)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2);

    if let Some(thread_id) = chat.thread_id {
        req = req.message_thread_id(teloxide::types::ThreadId(teloxide::types::MessageId(
            thread_id,
        )));
    }

    req.clone().send().await.map_err(|e| eyre::eyre!(e))
}

pub async fn edit_message(
    bot: &Bot,
    chat: Chat,
    message_id: MessageId,
    text: String,
) -> Result<Message, eyre::Report> {
    bot.edit_message_text(chat.id, message_id, text.clone())
        .disable_link_preview(true)
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .send()
        .await
        .map_err(|e| eyre::eyre!(e))
}

pub fn align_next_minute() -> Instant {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    let secs_left = Duration::from_secs(60 - now.as_secs() % 60);
    let nanos_left = secs_left - Duration::from_nanos(now.subsec_nanos() as u64);

    Instant::now() + nanos_left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_friday_test() {
        let date = NaiveDate::from_ymd_opt(2025, 5, 27).unwrap(); // Tuesday
        let next_friday_date = next_friday(date);
        assert_eq!(
            next_friday_date,
            NaiveDate::from_ymd_opt(2025, 5, 30).unwrap() // Friday
        );
    }
}
