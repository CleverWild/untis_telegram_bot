use std::{convert::Infallible, time::Duration};

use chrono::NaiveDate;
use shuttle_runtime::tokio;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::{ChatId, Request as _, Requester as _},
    Bot,
};
use tracing::Instrument;

use crate::utils::next_friday;
use crate::{diff_impl::Diff, PROD};
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

            let mut message = String::new();
            message.push_str("Changes in timetable:\n");

            let mut prev_date: Option<NaiveDate> = None;
            for (i, diff) in diffs.into_iter().enumerate() {
                let date = diff.date();
                if prev_date != Some(date) {
                    message.push_str(&format!(
                        "\nDate: {date} {week_day}\n",
                        week_day = date.format("%A")
                    ));

                    prev_date = Some(date);
                } else if i != 0 {
                    message.push_str("----------------\n");
                }

                message.push_str(&format_message(&diff));
            }

            if !message.is_empty() {
                let mut req = bot.send_message(*chat_id, message);

                if let Some(thread_id) = thread_id {
                    req = req.message_thread_id(teloxide::types::ThreadId(
                        teloxide::types::MessageId(*thread_id),
                    ));
                }

                if let Err(e) = req.send().await {
                    tracing::error!("Failed to send telegram message: {e}");
                }
            }
        }
    };

    // Save timetable to file
    if let Ok(json) = serde_json::to_string_pretty(&timetable) {
        if let Err(e) = std::fs::write(TIMETABLE_FILE, json) {
            tracing::warn!("Failed to save timetable to file: {e}");
        }
    }

    *prev = Some(timetable);
    Ok(())
}
