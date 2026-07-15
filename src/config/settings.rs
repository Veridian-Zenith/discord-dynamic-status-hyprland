use crate::constants;
use crate::logger::Logger;
use directories::ProjectDirs;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Clone)]
pub struct RpcRule {
    pub state: Option<String>,
    pub details: Option<String>,

    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub app_id: String,

    /// Fallback Discord asset key used for the large image when a class has no
    /// entry in `image_map`.
    #[serde(default)]
    pub default_large_image: Option<String>,

    /// Optional Discord asset key per window class (e.g. "kitty" -> "kitty").
    /// Anything not listed falls back to `default_large_image`.
    #[serde(default)]
    pub image_map: HashMap<String, String>,

    /// Optional pretty display name per class (e.g. "org.mozilla.firefox" ->
    /// "Firefox"). Unknown classes are auto-prettyfied from the raw class.
    #[serde(default)]
    pub name_map: HashMap<String, String>,

    /// Use the live window title as the details field. Defaults to true.
    #[serde(default = "default_true")]
    pub details_from_title: bool,
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load() -> Self {
        let proj_dirs = ProjectDirs::from(
            constants::QUALIFIER,
            constants::ORGANIZATION,
            constants::APP_NAME,
        )
        .expect("Failed to get application directory");

        let data_dir = proj_dirs.data_dir();

        if !data_dir.exists() {
            fs::create_dir_all(data_dir).expect("Failed to create data dir");
        }

        let config_path = data_dir.join("config.json");

        if !config_path.exists() {
            Logger::log("Config not found, creating default config...");

            let default_config = include_str!("default-config.json");

            fs::write(&config_path, default_config).expect("Failed to write default config");
        }

        let data = fs::read_to_string(&config_path).expect("Failed to read config.json");

        Logger::log("Config file loaded!");

        serde_json::from_str(&data).expect("Invalid config.json")
    }
}
