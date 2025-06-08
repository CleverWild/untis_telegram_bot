use shuttle_runtime::{tokio::task::JoinSet, SecretStore};
use teloxide::{prelude::ChatId, Bot};
use tracing::level_filters::LevelFilter;

mod diff_impl;
mod message;
mod message_formatter;
mod utils;
mod work;

use crate::work::working_loop;

// const PROD: bool = true;
const PROD: bool = !cfg!(debug_assertions);

/// My telegram DM
const DEBUG_TELEGRAM_CHAT: ChatId = ChatId(690963502);

type ShuttleUntis = Result<BotService, shuttle_runtime::Error>;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleUntis {
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

    let (chat_id, thread_id) = if PROD {
        (teloxide::prelude::ChatId(2476978824), Some(2))
    } else {
        (DEBUG_TELEGRAM_CHAT, None)
    };

    let whitelist = vec![work::WhitelistEntry {
        chat_id,
        thread_id,
        untis_school: "Gewerbliche Schule Waiblingen".to_string(),
        untis_login: "VABR2".to_string(),
        untis_password: "gswnVABR2DL".to_string(),
    }];

    let bot_service = BotService {
        whitelist,
        token: secret_store.get("bot_token").unwrap(),
    };

    Ok(bot_service)
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

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for BotService {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.launch().await;

        tracing::warn!("Bot finished");

        Ok(())
    }
}
