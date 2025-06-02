use shuttle_runtime::tokio::{self, task::JoinSet};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Request as _, Requester as _},
    Bot,
};

use crate::work::{working_loop, WhitelistEntry};

pub struct BotService {
    pub whitelist: Vec<WhitelistEntry>,
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
