use shuttle_runtime::SecretStore;
use tracing::level_filters::LevelFilter;

mod bot_service;
mod diff_impl;
mod message_formatter;
mod utils;
mod work;

use bot_service::BotService;
use work::WhitelistEntry;

const PROD: bool = false;
// const PROD: bool = !cfg!(debug_assertions);

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
        (teloxide::prelude::ChatId(690963502), None) // My telegram DM
    };

    let whitelist = vec![WhitelistEntry {
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
