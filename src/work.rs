use std::{convert::Infallible, time::Duration};

use chrono::NaiveDate;
use shuttle_runtime::tokio;
use teloxide::{
    Bot,
    payloads::SendMessageSetters,
    prelude::{ChatId, Request as _, Requester as _},
};
use tracing::Instrument;

use crate::{DEBUG_TELEGRAM_CHAT, message_formatter::core::apply_debug_info, utils::next_friday};
use crate::{PROD, diff_impl::Diff};
use crate::{message_formatter::format_message, utils::sort_diffs};

const TIMETABLE_FILE: &str = "timetable.json";

#[derive(Debug, Clone)]
pub struct WhitelistEntry {
    pub chat_id: ChatId,
    pub thread_id: Option<i32>,
    pub untis_school: String,
    pub untis_login: String,
    pub untis_password: String,
}

#[tracing::instrument(skip_all, fields(%class = whitelist.untis_login))]
pub async fn working_loop(bot: Bot, whitelist: WhitelistEntry) -> Result<Infallible, eyre::Report> {
    let WhitelistEntry {
        untis_school,
        untis_login,
        untis_password,
        chat_id,
        ..
    } = &whitelist;

    let span = tracing::info_span!("preparation");

    tracing::info!(parent: &span, "Starting fetch");

    let school = untis::schools::get_by_name(untis_school.as_str())
        .instrument(span.clone())
        .await
        .inspect_err(|e| tracing::error!("Failed to get school '{untis_school}': {e}"))
        .unwrap();

    tracing::info!(parent: &span, "school received: {:?}", school);

    let mut client = school
        .client_login(untis_login.as_str(), untis_password.as_str())
        .instrument(span.clone())
        .await
        .unwrap();

    let mut prev: Option<Vec<untis::Lesson>> = std::fs::read_to_string(TIMETABLE_FILE)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .or_else(|| {
            tracing::warn!("Failed to load previous timetable from file");
            None
        });

    const DURATION: Duration = Duration::from_secs(60);
    let mut interval = tokio::time::interval(DURATION);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
    interval.tick().await;

    for i in 0u32.. {
        let span = tracing::info_span!("iteration", i);

        if !PROD && i % 10 == 0 {
            bot.send_message(*chat_id, format!("{untis_login} is alive"))
                .send()
                .await?;
        }

        let start = tokio::time::Instant::now();

        work(&bot, &mut client, &whitelist, &mut prev)
            .instrument(span.clone())
            .await?;

        let json = serde_json::to_string_pretty(&prev).expect("Failed to serialize timetable");
        if let Err(e) = std::fs::write(TIMETABLE_FILE, json) {
            tracing::error!(parent: &span, "Failed to save timetable to file: {e}");
        }

        // Warn if time to work is to long
        let elapsed = start.elapsed();
        if elapsed > DURATION {
            tracing::warn!(parent: &span, "Iteration took longer ({elapsed:?}) than the interval");
        } else {
            tracing::trace!(parent: &span, "Iteration took {elapsed:?}");
        }

        interval.tick().await;
    }

    #[allow(dead_code)]
    const YEARS_TO_OVERFLOW: u64 = u32::MAX as u64 / (365 * 24 * 60 * 60) * DURATION.as_secs();
    unreachable!("Integer overflow is unlikely here, but possible, see <YEARS_TO_OVERFLOW> const");
}

#[tracing::instrument(skip_all)]
async fn work(
    bot: &Bot,
    client: &mut untis::Client,
    entry: &WhitelistEntry,
    prev: &mut Option<Vec<untis::Lesson>>,
) -> Result<(), eyre::Report> {
    let WhitelistEntry {
        chat_id, thread_id, ..
    } = entry;

    tracing::info!("Fetching timetable");

    let timetable = client
        .own_timetable_until(&untis::Date({
            next_friday(chrono::Local::now().date_naive()) + chrono::Duration::days(7)
        }))
        .await?;

    if let Some(prev) = prev {
        let mut diffs = Diff::find(prev, &timetable);

        if !diffs.is_empty() {
            sort_diffs(&mut diffs);

            tracing::info!("Diff was found: {:#?}", diffs);

            let mut prod_message = "Changes in timetable:\n".to_string();
            // Extended variation of prod_message but with debug information
            let mut debug_message = prod_message.clone();

            let mut prev_date: Option<NaiveDate> = None;
            for (i, diff) in diffs.into_iter().enumerate() {
                let date = diff.date();
                let mut separator = String::new();
                if prev_date != Some(date) {
                    separator.push_str(&format!(
                        "\nDate: {date} {week_day}\n",
                        week_day = date.format("%A")
                    ));

                    prev_date = Some(date);
                } else if i != 0 {
                    separator.push_str("----------------\n");
                }

                prod_message.push_str(&separator);
                debug_message.push_str(&separator);

                let formatted = format_message(&diff);
                prod_message.push_str(&formatted.to_string());

                let mut debug_formatted = formatted;
                apply_debug_info(&mut debug_formatted, entry, &diff);
                debug_message.push_str(&debug_formatted.to_string());
            }

            if PROD && !prod_message.is_empty() {
                send_message(bot, *chat_id, prod_message, *thread_id).await;
            }

            if !debug_message.is_empty() {
                send_message(bot, DEBUG_TELEGRAM_CHAT, debug_message, None).await;
            }
        }
    };

    // Save timetable to file
    match serde_json::to_string_pretty(&timetable) {
        Ok(json) => {
            if let Err(e) = std::fs::write(TIMETABLE_FILE, json) {
                tracing::warn!("Failed to save timetable to file: {e}");
            }
        }
        Err(e) => {
            tracing::error!("Failed to serialize timetable: {e}");
        }
    }

    *prev = Some(timetable);
    Ok(())
}

async fn send_message(bot: &Bot, chat_id: ChatId, message: String, thread_id: Option<i32>) {
    let mut req = bot.send_message(chat_id, message);

    if let Some(thread_id) = thread_id {
        req = req.message_thread_id(teloxide::types::ThreadId(teloxide::types::MessageId(
            thread_id,
        )));
    }

    if let Err(e) = req.clone().send().await {
        // todo! after integration DB edit the thread id after creating a new topic
        // if let teloxide::RequestError::Api(teloxide::ApiError::Unknown(ref str)) = e
        //     && str.contains("message thread not found")
        // {
        //     let name = format!("{} Notification", entry.display_name);
        //     bot.create_forum_topic(chat_id, name, icon_color, icon_custom_emoji_id)
        // } else {
        tracing::error!("Failed to send telegram message: {e}");
        // }
    }
}
