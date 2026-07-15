mod config;
mod constants;
mod discord;
mod hyprland;
mod logger;
mod rules;

use discord::rpc::DiscordRpc;
use hyprland::events::listen_active_window;
use logger::Logger;

fn main() {
    Logger::log(&format!(
        "Starting application v{} ({} / {})",
        constants::VERSION,
        constants::detect_os(),
        std::env::consts::ARCH
    ));

    let config = config::Config::load();

    Logger::log("Config loaded successfully!");

    let mut rpc = DiscordRpc::new(&config.app_id);

    rpc.connect();

    Logger::log("Connected to Discord successfully!");

    listen_active_window(|class, title| {
        let presence = rules::build_presence(&config, &class, &title);

        rpc.update(&presence, &title);
    });
}
