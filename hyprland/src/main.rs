use common::config::Config;
use common::constants;
use common::discord::rpc::DiscordRpc;
use common::logger::Logger;
use common::rules;
use std::env;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;

fn main() {
    Logger::init_logger(constants::HYPRLAND_APP_NAME);

    Logger::log(&format!(
        "Starting application v{} ({} / {})",
        constants::VERSION,
        constants::detect_os(),
        std::env::consts::ARCH
    ));

    let config = Config::load(
        constants::HYPRLAND_APP_NAME,
        include_str!("../../common/src/config/default-config.json"),
    );

    Logger::log("Config loaded successfully!");

    let mut rpc = DiscordRpc::new(&config.app_id);

    rpc.connect();

    Logger::log("Connected to Discord successfully!");

    listen_active_window(|class, title| {
        let presence = rules::build_presence(&config, &class, &title, "Hyprland");

        rpc.update(&presence, &title);
    });
}

fn listen_active_window<F>(mut handler: F)
where
    F: FnMut(String, String),
{
    let runtime = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE not set");

    Logger::log(&format!("Runtime: {}, Signature: {}", runtime, sig));

    let path = format!("{runtime}/hypr/{sig}/.socket2.sock");
    let stream = UnixStream::connect(path).expect("Failed to connect to Hyprland socket");

    let reader = BufReader::new(stream);

    #[allow(clippy::lines_filter_map_ok)]
    for line in reader.lines().filter_map(Result::ok) {
        if let Some(data) = line.strip_prefix("activewindow>>") {
            let mut parts = data.splitn(2, ',');

            let class = parts.next().unwrap_or("").to_string();
            let title = parts.next().unwrap_or("").to_string();

            Logger::log(&format!(
                "Current class: {}, current title: {}",
                class, title
            ));

            handler(class, title);
        }
    }
}
