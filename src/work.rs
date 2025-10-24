use std::{convert::Infallible, time::Duration};

use chrono::NaiveDate;
use teloxide::{
    Bot,
    prelude::{ChatId, Request as _, Requester as _},
    types::MessageId,
};
use tokio::time::Instant;
use tracing::Instrument;

use crate::{
    DEBUG_TELEGRAM_CHAT, message,
    message_formatter::core::apply_debug_info,
    utils::{edit_message, next_friday, send_message},
};
use crate::{IS_PROD, diff_impl::Diff};
use crate::{message_formatter::format_message, utils::sort_diffs};

const TIMETABLE_FILE: &str = "timetable.json";

#[derive(Debug, Clone, Copy)]
pub struct Chat {
    pub id: ChatId,
    pub thread_id: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub uptime_since: Instant,
    pub notification_chat: Chat,
    pub status_chat: Chat,
    pub status_msg: Option<MessageId>,
    pub untis_school: String,
    pub untis_login: String,
    pub untis_password: String,
    pub target_class_name: Option<String>,
    pub task_name: String,
}

#[tracing::instrument(skip_all, fields(%task = whitelist.task_name))]
pub async fn working_loop(bot: Bot, whitelist: TaskInfo) -> Result<Infallible, eyre::Report> {
    let TaskInfo {
        untis_school,
        untis_login,
        untis_password,
        notification_chat,
        task_name,
        status_msg,
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

    let mut status_msg: Option<MessageId> = *status_msg;
    let mut prev: Option<Vec<untis::Lesson>> = match std::fs::read_to_string(TIMETABLE_FILE) {
        Ok(content) => match serde_json::from_str::<Vec<_>>(&content) {
            Ok(timetable) => {
                tracing::info!(parent: &span, "Restored timetable from file with {} lessons", timetable.len());
                Some(timetable)
            }
            Err(e) => {
                tracing::warn!(parent: &span, "Failed to parse timetable from file: {e}");
                None
            }
        },
        Err(e) => {
            tracing::info!(parent: &span, "No previous timetable file found: {e}");
            None
        }
    };

    const DURATION: Duration = Duration::from_secs(60);

    let mut interval = tokio::time::interval_at(crate::utils::align_next_minute(), DURATION);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    for i in 0u32.. {
        let span = tracing::info_span!("iteration", i);

        if !IS_PROD && i % 10 == 0 {
            bot.send_message(notification_chat.id, format!("{task_name} is alive"))
                .send()
                .await?;
        }

        let start = tokio::time::Instant::now();

        work(&bot, &mut client, &whitelist, &mut prev)
            .instrument(span.clone())
            .await?;

        update_status(
            &bot,
            &mut client,
            &whitelist,
            &mut status_msg,
            prev.as_ref()
                .expect("Function 'work' above should set 'prev'"),
        )
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
    entry: &TaskInfo,
    prev: &mut Option<Vec<untis::Lesson>>,
) -> Result<(), eyre::Report> {
    let TaskInfo {
        notification_chat,
        target_class_name,
        ..
    } = entry;

    tracing::info!("Fetching timetable");

    let date =
        untis::Date(next_friday(chrono::Local::now().date_naive()) + chrono::Duration::days(14));
    let timetable = match target_class_name {
        Some(class_name) => {
            let classes = client.classes().await?;
            let found_class = classes.into_iter().find(|class| class.name == *class_name);
            let id = found_class.map(|class| class.id).ok_or_else(|| {
                eyre::eyre!("Failed to find class with name {:?}", target_class_name)
            })?;

            client
                .timetable_until(&id, &untis::ElementType::Class, &date)
                .await?
        }
        None => client.own_timetable_until(&date).await?,
    };

    if let Some(prev) = prev {
        let mut diffs = Diff::find(prev, &timetable);

        if !diffs.is_empty() {
            sort_diffs(&mut diffs);

            tracing::info!("Diff was found: {:#?}", diffs);

            let mut message = message::LabeledMessage::new();
            let mut prev_date: Option<NaiveDate> = None;

            for (i, diff) in diffs.into_iter().enumerate() {
                let date = diff.date();

                if let Some(separator) = if prev_date != Some(date) {
                    prev_date = Some(date);
                    Some(format!("\nDate: {date} {}\n", date.format("%A")))
                } else if i != 0 {
                    Some("----------------\n".to_string())
                } else {
                    None
                } {
                    message.push(separator);
                }

                let formatted = format_message(&diff).into_labeled_message();

                // Add normal view
                message.extend(formatted.filter_normal());

                // Add debug view with additional debug info
                message.debug_ln(|msg| {
                    msg.extend(formatted);
                    apply_debug_info(msg, entry, &diff)
                });
            }

            if IS_PROD {
                // Send message to production target
                tracing::info!(
                    "Sending to chat `{}` with topic `{:?}` message:\n{}",
                    notification_chat.id,
                    notification_chat.thread_id,
                    message
                );
                send_message(bot, *notification_chat, message.filter_normal().to_string()).await?;
            }
            // Send message to debug target
            send_message(bot, DEBUG_TELEGRAM_CHAT, message.to_string()).await?;
        }
    }

    if !IS_PROD {
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
    }

    *prev = Some(timetable);
    Ok(())
}

async fn update_status(
    bot: &Bot,
    client: &mut untis::Client,
    entry: &TaskInfo,
    status_msg_id: &mut Option<MessageId>,
    timetable: &[untis::Lesson],
) -> Result<(), eyre::Report> {
    let TaskInfo { status_chat, .. } = entry;

    let today = chrono::Local::now().date_naive();

    // Filter out expired homeworks based on timetable
    // Only filter homeworks that are due TODAY and the lesson has already passed
    let homeworks: Vec<_> = client
        .homeworks_data()
        .await?
        .into_homeworks()
        .into_iter()
        .filter(|hw| {
            // Keep all future/past homeworks as-is
            if hw.due_date.0 != today {
                return true;
            }

            // For today's homeworks, check if the subject lesson already occurred
            let subject_occurred_today =
                timetable
                    .iter()
                    .filter(|l| l.date.0 == today)
                    .any(|lesson| {
                        // Check if this lesson is for the same subject
                        let is_same_subject = lesson
                            .subjects
                            .iter()
                            .any(|subj| subj.name == hw.lesson.subject);

                        // Check if this lesson is taught by the same teacher
                        let is_same_teacher = lesson
                            .teachers
                            .iter()
                            .any(|teacher| teacher.name == hw.teacher.name);

                        is_same_subject && is_same_teacher
                    });

            // Keep homework only if the subject lesson hasn't occurred today
            !subject_occurred_today
        })
        .collect();

    let status_message =
        crate::status::StatusMessage::new(homeworks, timetable, entry.uptime_since).into_message();

    if let Some(message_id) = status_msg_id {
        if let Err(e) =
            edit_message(bot, *status_chat, *message_id, status_message.to_string()).await
        {
            tracing::warn!("Failed to edit status message: {e}");
            *status_msg_id = None;
        }
    } else {
        tracing::warn!("Re-sending status message");
        let msg = send_message(bot, *status_chat, status_message.to_string()).await?;
        status_msg_id.replace(msg.id);
    }

    Ok(())
}
