use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use teloxide::{Bot, prelude::ChatId};
use tokio::task::JoinSet;
use tracing::level_filters::LevelFilter;

mod diff_impl;
mod message;
mod message_formatter;
mod status;
mod utils;
mod work;

use crate::work::{Chat, WorkerContext, working_loop};

const IS_PROD: bool = !cfg!(debug_assertions);

/// My telegram DM
const DEBUG_TELEGRAM_CHAT: Chat = Chat {
    id: ChatId(690963502),
    thread_id: None,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // .without_time()
        .pretty()
        .with_ansi(!IS_PROD) // Disable ANSI color codes in production for cleaner logs
        .with_line_number(true)
        // .compact()
        .init();

    let token = {
        // Try to load from environment variable first, then from file
        std::env::var("BOT_TOKEN").unwrap_or_else(|_| {
            let config = config::Config::builder()
                .add_source(config::File::with_name("Secrets.toml"))
                .build()
                .expect("Failed to load Secrets.toml or BOT_TOKEN env var");
            config
                .get::<String>("bot_token")
                .expect("Set BOT_TOKEN env var or bot_token in Secrets.toml")
        })
    };

    let bot = Bot::new(token);

    tracing::info!("Initializing database...");
    if let Err(e) = db::init_db() {
        tracing::error!("Failed to initialize database: {e}");
        panic!("Cannot continue without database");
    }
    tracing::info!("Database initialized");
    let mut join_handles = JoinSet::new();
    let mut running_tasks: HashMap<db::Uuid, tokio::task::AbortHandle> = HashMap::new();
    tracing::info!("Starting observer loop...");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tracing::info!("Fetching bot states...");

        // Fetch all bot states
        let bot_states = match db::models::BotTask::get_all() {
            Ok(states) => states,
            Err(e) => {
                tracing::error!("Failed to fetch bot states: {e}");
                continue;
            }
        };

        tracing::debug!("Found {} bot state(s) in database", bot_states.len());

        // Check for new or missing states
        let current_state_ids: HashSet<_> = bot_states.iter().map(|s| s.id).collect();

        // Remove tasks for states that no longer exist
        running_tasks.retain(|id, abort_handle| {
            if !current_state_ids.contains(id) {
                tracing::warn!("Bot state {id} no longer exists, stopping worker");
                abort_handle.abort();
                false
            } else {
                true
            }
        });

        // Start workers for new states
        for task in bot_states {
            if running_tasks.contains_key(&task.id) {
                continue; // Already running
            }

            tracing::info!("Starting worker for bot state: {}", task.task_name);

            let school = match untis::schools::get_by_name(task.untis_school.as_str()).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        "Failed to get school '{}' for task {}: {e}",
                        task.untis_school,
                        task.task_name
                    );
                    continue;
                }
            };

            let untis_client = match school
                .client_login(task.untis_login.as_str(), task.untis_password.as_str())
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to login to Untis for task {}: {e}", task.task_name);
                    continue;
                }
            };

            // Wrap the future to return identifying info along with the result
            let state_id = task.id;
            let task_name = task.task_name.clone();

            let ctx = WorkerContext {
                bot: bot.clone(),
                task,
                untis_client,
                engaged_at: chrono::Utc::now(),
            };

            let abort_handle =
                join_handles.spawn(async move { (state_id, task_name, working_loop(ctx).await) });
            running_tasks.insert(state_id, abort_handle);
        }

        loop {
            tokio::select! {
                _ = interval.tick() => break,
                Some(result) = join_handles.join_next() => {
                    match result {
                        // Task returned an application error
                        Ok((state_id, task_name, Err(e))) => {
                            running_tasks.remove(&state_id);
                            tracing::error!(
                                task = %task_name,
                                error = %e,
                                "Worker encountered an error, will restart on next cycle"
                            );
                            interval.reset_after(Duration::from_secs(2));
                        }
                        // Join error: cancelled or panicked
                        Err(join_err) => {
                            if join_err.is_panic() {
                                tracing::error!("Worker panicked: {join_err}");
                                interval.reset_after(Duration::from_secs(2));
                            }
                            // Cancelled tasks are expected
                        }
                    }
                }
            }
        }
    }
}

// fn make_whitelist() -> work::TaskInfo {
//     // Load from environment variables with fallback to defaults
//     let untis_school =
//         std::env::var("UNTIS_SCHOOL").unwrap_or_else(|_| "KS-Waiblingen".to_string());
//     let untis_login = std::env::var("UNTIS_LOGIN").unwrap_or_else(|_| "BrovkoOle".to_string());
//     let untis_password =
//         std::env::var("UNTIS_PASSWORD").unwrap_or_else(|_| "N6C4csN&^*a7vW".to_string());

//     if IS_PROD {
//         work::TaskInfo {
//             uptime_since: Instant::now(),
//             notification_chat: Chat {
//                 // For supergroups/channels the real chat id = "-100" + <numeric from /c/>
//                 id: ChatId(-1002951933538),
//                 thread_id: Some(2),
//             },
//             status_chat: Chat {
//                 id: ChatId(-1002951933538),
//                 thread_id: Some(231),
//             },
//             status_msg_id: Some(MessageId(253)), // https://t.me/c/2951933538/231/253
//             untis_school,
//             untis_login,
//             untis_password,
//             target_class_name: None,
//             task_name: "VABO1".to_string(),
//         }
//     } else {
//         work::TaskInfo {
//             uptime_since: Instant::now(),
//             notification_chat: DEBUG_TELEGRAM_CHAT,
//             status_chat: DEBUG_TELEGRAM_CHAT,
//             status_msg_id: None,
//             untis_school,
//             untis_login,
//             untis_password,
//             target_class_name: None,
//             task_name: "VABO1".to_string(),
//         }
//     }
// }
