mod toplevel;

use common::config::Config;
use common::constants;
use common::discord::rpc::DiscordRpc;
use common::logger::Logger;
use common::rules;
use std::collections::HashMap;
use std::sync::mpsc;
use wayland_client::{
    Connection, Proxy, QueueHandle,
    protocol::wl_display::WlDisplay,
    protocol::wl_registry::{self, WlRegistry},
};

fn main() {
    Logger::init_logger(constants::COSMIC_APP_NAME);

    Logger::log(&format!(
        "Starting application v{} ({} / {})",
        constants::VERSION,
        constants::detect_os(),
        std::env::consts::ARCH
    ));

    let config = Config::load(constants::COSMIC_APP_NAME);

    Logger::log("Config loaded successfully!");

    let mut rpc = DiscordRpc::new(&config.app_id);

    rpc.connect();

    Logger::log("Connected to Discord successfully!");

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland display");

    Logger::log("Connected to Wayland display");

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let display = conn.display();

    let _registry = display.get_registry(&qh, ());

    Logger::log("Requesting toplevel protocols, listening for window changes...");

    let config = Arc::new(config);
    let (tx, rx) = mpsc::channel::<(String, String)>();

    let wayland_config = config.clone();
    std::thread::Builder::new()
        .name("wayland-events".into())
        .spawn(move || {
            let mut state = AppState::new(wayland_config, tx);
            loop {
                if let Err(e) = event_queue.blocking_dispatch(&mut state) {
                    Logger::log(&format!("Error in Wayland event dispatch: {:?}", e));
                    break;
                }
            }
        })
        .expect("Failed to spawn Wayland thread");

    loop {
        if let Ok((class, title)) = rx.recv() {
            Logger::log(&format!(
                "Current class: {}, current title: {}",
                class, title
            ));

            let presence = rules::build_presence(&config, &class, &title, "COSMIC");

            rpc.update(&presence, &title);
        }
    }
}

use std::sync::Arc;

pub struct ToplevelInfo {
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub activated: bool,
}

pub struct ToplevelState {
    pub toplevels: HashMap<u32, ToplevelInfo>,
    active_id: Option<u32>,
    focus_change: Option<(String, String)>,
}

impl ToplevelState {
    fn new() -> Self {
        Self {
            toplevels: HashMap::new(),
            active_id: None,
            focus_change: None,
        }
    }

    pub fn upsert_toplevel(&mut self, id: u32) -> &mut ToplevelInfo {
        self.toplevels.entry(id).or_insert_with(|| ToplevelInfo {
            app_id: None,
            title: None,
            activated: false,
        })
    }

    pub fn check_focus(&mut self) {
        let new_active = self
            .toplevels
            .iter()
            .find(|(_, info)| info.activated)
            .map(|(id, _)| *id);

        if new_active != self.active_id {
            self.active_id = new_active;

            if let Some(id) = new_active
                && let Some(info) = self.toplevels.get(&id)
            {
                let class = info.app_id.clone().unwrap_or_default();
                let title = info.title.clone().unwrap_or_default();

                if !class.is_empty() {
                    self.focus_change = Some((class, title));
                }
            }
        }
    }
}

pub struct AppState {
    pub toplevel_state: ToplevelState,
    pub cosmic_toplevel_manager: Option<
        cosmic_protocols::toplevel_info::v1::client::zcosmic_toplevel_info_v1::ZcosmicToplevelInfoV1,
    >,
    pub config: Arc<Config>,
    pub tx: mpsc::Sender<(String, String)>,
}

impl AppState {
    fn new(config: Arc<Config>, tx: mpsc::Sender<(String, String)>) -> Self {
        Self {
            toplevel_state: ToplevelState::new(),
            cosmic_toplevel_manager: None,
            config,
            tx,
        }
    }

    pub fn check_focus_and_notify(&mut self) {
        self.toplevel_state.check_focus();
        if let Some(change) = self.toplevel_state.take_focus_change() {
            let _ = self.tx.send(change);
        }
    }
}

impl ToplevelState {
    pub fn take_focus_change(&mut self) -> Option<(String, String)> {
        self.focus_change.take()
    }
}

impl wayland_client::Dispatch<WlDisplay, ()> for AppState {
    fn event(
        _state: &mut Self,
        _proxy: &WlDisplay,
        _event: <WlDisplay as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "zcosmic_toplevel_info_v1"
        {
            Logger::log(&format!(
                "Found zcosmic_toplevel_info_v1 (name={}, version={})",
                name, version
            ));
            let proxy = registry.bind(name, version.min(1), qhandle, ());
            state.cosmic_toplevel_manager = Some(proxy);
        }
    }
}
