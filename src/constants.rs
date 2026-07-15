pub const QUALIFIER: &str = "ru";
pub const ORGANIZATION: &str = "Kazuha046";
pub const APP_NAME: &str = "Dynamic-DRPC-Hyprland";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_os = "linux")]
pub const OS: &str = "linux";
#[cfg(target_os = "macos")]
pub const OS: &str = "macos";
#[cfg(target_os = "windows")]
pub const OS: &str = "windows";
