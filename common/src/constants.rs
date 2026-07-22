pub const QUALIFIER: &str = "ru";
pub const ORGANIZATION: &str = "Kazuha046";

pub const HYPRLAND_APP_NAME: &str = "Dynamic-DRPC-Hyprland";
pub const COSMIC_APP_NAME: &str = "Dynamic-DRPC-COSMIC";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn detect_os() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let mut name = None;
            let mut id = None;
            let mut version_id = None;

            for line in content.lines() {
                if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                    return value.trim_matches('"').to_string();
                }
                if let Some(value) = line.strip_prefix("NAME=") {
                    name = Some(value.trim_matches('"').to_string());
                }
                if let Some(value) = line.strip_prefix("ID=") {
                    id = Some(value.trim_matches('"').to_string());
                }
                if let Some(value) = line.strip_prefix("VERSION_ID=") {
                    version_id = Some(value.trim_matches('"').to_string());
                }
            }

            if let Some(n) = name {
                return n;
            }
            if let Some(i) = id {
                let mut result = i;
                if let Some(v) = version_id {
                    result.push(' ');
                    result.push_str(&v);
                }
                return result;
            }
        }
        "Linux".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        std::env::consts::OS.to_string()
    }
}
