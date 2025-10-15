use teloxide::{Bot, prelude::ChatId, types::MessageId};
use tokio::task::JoinSet;
use tracing::level_filters::LevelFilter;

mod diff_impl;
mod message;
mod message_formatter;
mod status;
mod utils;
mod work;

use crate::work::{Chat, working_loop};

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
        .without_time()
        .pretty()
        .compact()
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

    let bot_service = BotService {
        whitelist: vec![make_whitelist()],
        token,
    };

    bot_service.launch().await;
}

pub struct BotService {
    pub whitelist: Vec<work::WhitelistEntry>,
    pub token: String,
}

impl BotService {
    pub async fn launch(&self) {
        let bot = Bot::new(&self.token);

        let mut join_handles = JoinSet::new();

        for entry in self.whitelist.iter().cloned() {
            let bot = bot.clone();
            join_handles.spawn(working_loop(bot, entry));
        }

        while let Some(result) = join_handles.join_next().await {
            match result {
                Ok(Ok(_)) => unreachable!(),
                Ok(Err(e)) => tracing::error!("Worker encountered an error: {e}"),
                Err(e) => tracing::error!("Worker panicked: {e}"),
            }
        }
    }
}

fn make_whitelist() -> work::WhitelistEntry {
    // Load from environment variables with fallback to defaults
    let untis_school = std::env::var("UNTIS_SCHOOL").unwrap_or_else(|_| "KS-Waiblingen".to_string());
    let untis_login = std::env::var("UNTIS_LOGIN").unwrap_or_else(|_| "BrovkoOle".to_string());
    let untis_password = std::env::var("UNTIS_PASSWORD").unwrap_or_else(|_| "N6C4csN&^*a7vW".to_string());
    
    if IS_PROD {
        work::WhitelistEntry {
            notification_chat: Chat {
                // For supergroups/channels the real chat id = "-100" + <numeric from /c/>
                id: ChatId(-1002951933538),
                thread_id: Some(2),
            },
            status_chat: Chat {
                id: ChatId(-1002951933538),
                thread_id: Some(231),
            },
            status_msg: Some(MessageId(253)), // https://t.me/c/2951933538/231/253
            untis_school,
            untis_login,
            untis_password,
            target_class_name: None,
            task_name: "VABO1".to_string(),
        }
    } else {
        work::WhitelistEntry {
            notification_chat: DEBUG_TELEGRAM_CHAT,
            status_chat: DEBUG_TELEGRAM_CHAT,
            status_msg: None,
            untis_school,
            untis_login,
            untis_password,
            target_class_name: None,
            task_name: "VABO1".to_string(),
        }
    }
}
