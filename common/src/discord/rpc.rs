use crate::config::RpcRule;
use crate::logger::Logger;
use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};

pub struct DiscordRpc {
    client: DiscordIpcClient,
}

impl DiscordRpc {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: DiscordIpcClient::new(client_id),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        self.client
            .connect()
            .map_err(|e| format!("Failed to connect to Discord: {}", e))
    }

    fn build_activity(rule: &RpcRule) -> activity::Activity<'_> {
        let mut act = activity::Activity::new();

        if let Some(state) = &rule.state {
            act = act.state(state);
        }

        if let Some(details) = &rule.details {
            act = act.details(details);
        }

        if rule.large_image.is_some() || rule.small_image.is_some() {
            let mut assets = activity::Assets::new();

            if let Some(v) = &rule.large_image {
                assets = assets.large_image(v);
            }
            if let Some(v) = &rule.large_text {
                assets = assets.large_text(v);
            }
            if let Some(v) = &rule.small_image {
                assets = assets.small_image(v);
            }
            if let Some(v) = &rule.small_text {
                assets = assets.small_text(v);
            }

            act = act.assets(assets);
        }

        act
    }

    pub fn update(&mut self, rule: &RpcRule, title: &str) {
        Logger::log(&format!(
            "[RPC] class_title={:?}, state={:?}, details={:?}, large_image={:?}, large_text={:?}, small_image={:?}, small_text={:?}",
            title,
            rule.state,
            rule.details,
            rule.large_image,
            rule.large_text,
            rule.small_image,
            rule.small_text
        ));

        let act = Self::build_activity(rule);

        if let Err(e) = self.client.set_activity(act) {
            Logger::log(&format!(
                "[RPC] Failed to set activity: {}. Attempting reconnect...",
                e
            ));
            if let Err(re) = self.connect() {
                Logger::log(&format!("[RPC] Reconnect failed: {}", re));
                return;
            }
            let act = Self::build_activity(rule);
            if let Err(e2) = self.client.set_activity(act) {
                Logger::log(&format!("[RPC] Retry also failed: {}", e2));
            }
        }
    }
}
