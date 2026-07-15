use crate::config::{Config, RpcRule};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Discord enforces a 128 char limit on text fields.
const MAX_FIELD_LEN: usize = 128;

/// Derived, class-specific data that never changes between windows of the same
/// application. Cached so we don't re-run `pretty_name` / map lookups on every
/// active-window event.
#[derive(Clone)]
struct ClassInfo {
    name: String,
    large_image: Option<String>,
}

fn class_cache() -> &'static Mutex<HashMap<String, ClassInfo>> {
    static CACHE: OnceLock<Mutex<HashMap<String, ClassInfo>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Build the rich-presence payload dynamically from the active window instead
/// of looking it up in a static per-class table.
pub fn build_presence(config: &Config, class: &str, title: &str) -> RpcRule {
    let info = resolve_class(config, class);

    let details = if config.details_from_title {
        Some(truncate(title))
    } else {
        None
    };

    RpcRule {
        state: Some(format!("Using {}", info.name)),
        details,
        large_image: info.large_image,
        large_text: Some(info.name),
        small_image: Some("hyprland".to_string()),
        small_text: Some("Hyprland".to_string()),
    }
}

/// Resolve (and cache) the per-class derived info.
fn resolve_class(config: &Config, class: &str) -> ClassInfo {
    if let Some(info) = class_cache().lock().unwrap().get(class).cloned() {
        return info;
    }

    let info = ClassInfo {
        name: pretty_name(config, class),
        large_image: config
            .image_map
            .get(class)
            .cloned()
            .or_else(|| config.default_large_image.clone()),
    };

    class_cache()
        .lock()
        .unwrap()
        .insert(class.to_string(), info.clone());

    info
}

/// Turn a raw window class into a nice display name.
/// "org.mozilla.firefox" -> "Firefox", "kitty" -> "Kitty".
fn pretty_name(config: &Config, class: &str) -> String {
    if let Some(name) = config.name_map.get(class) {
        return name.clone();
    }

    class
        .rsplit('.')
        .next()
        .unwrap_or(class)
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate(s: &str) -> String {
    let s = s.trim();
    if s.chars().count() <= MAX_FIELD_LEN {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(MAX_FIELD_LEN - 1).collect();
        format!("{}…", truncated)
    }
}
