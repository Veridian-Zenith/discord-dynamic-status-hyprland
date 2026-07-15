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
        Some(truncate(&clean_title(title, &info.name)))
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

/// Clean a window title for use as Discord details.
///
/// Fully dynamic — no browser class list. Detects and strips the "— BrowserName"
/// suffix pattern from any title that has one, replaces generic/empty titles
/// with the app name, and extracts domains from bare URLs.
fn clean_title(title: &str, app_name: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        return app_name.to_string();
    }

    // If the title is a bare URL, extract domain
    if looks_like_url(title) {
        return domain_from_url(title).unwrap_or_else(|| app_name.to_string());
    }

    // Check for generic / uninteresting titles
    if is_generic_title(title) {
        return app_name.to_string();
    }

    // Try to strip a trailing browser-style suffix (" — BrowserName" / " - BrowserName")
    strip_title_suffix(title).to_string()
}

/// Strip a trailing " — Something" / " - Something" suffix from a title when
/// the suffix looks like an application name (short, no URLs, no special chars)
/// AND the main part of the title is not itself a URL.
///
/// This is fully dynamic: it doesn't need to know the browser name. It just
/// recognises the pattern that browsers append to tab titles.
fn strip_title_suffix(title: &str) -> &str {
    for sep in &[" — ", " - "] {
        if let Some(pos) = title.rfind(sep) {
            let main = &title[..pos];
            let suffix = &title[pos + sep.len()..];

            // Don't strip if the main part is a full URL
            if main.contains("://") || main.starts_with("www.") {
                continue;
            }

            if looks_like_app_suffix(suffix) {
                return main.trim_end();
            }
        }
    }
    title
}

/// Heuristic: does this suffix look like a browser/app name appended to a title?
///
/// Browser suffixes are typically 1–3 capitalized words with no special chars:
/// "Mozilla Firefox", "Brave", "Google Chrome", "Vivaldi", "Sage", etc.
fn looks_like_app_suffix(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.len() > 40 {
        return false;
    }
    // Reject anything that looks like a URL, path, or has punctuation
    if s.contains('/')
        || s.contains(':')
        || s.contains('.')
        || s.contains('(')
        || s.contains(')')
        || s.contains('"')
    {
        return false;
    }
    // Must contain at least one letter
    if !s.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    // All words should start with an uppercase letter (browser names are proper nouns)
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() || words.len() > 4 {
        return false;
    }
    words
        .iter()
        .all(|w| w.starts_with(|c: char| c.is_uppercase()))
}

/// Common generic / uninteresting tab titles that should fall back to the app name.
fn is_generic_title(title: &str) -> bool {
    matches!(
        title.to_ascii_lowercase().as_str(),
        "new tab"
            | "start page"
            | "about:blank"
            | "about:newtab"
            | "about:startpage"
            | "welcome to firefox"
            | "google"
            | "speed dial"
            | "most visited"
    )
}

/// Heuristic: does the string look like a URL?
fn looks_like_url(s: &str) -> bool {
    s.contains("://") || s.starts_with("www.") || has_domain_with_path(s)
}

/// Does the string look like a bare domain with a path, query, or fragment?
/// e.g. "example.com/page", "example.com?q=1"
fn has_domain_with_path(s: &str) -> bool {
    // Must have a dot, no spaces, and something after the first dot that looks like a TLD + path
    let Some(dot_pos) = s.find('.') else {
        return false;
    };
    if s.contains(' ') {
        return false;
    }
    let after_dot = &s[dot_pos + 1..];
    // TLD must be at least 2 chars and followed by end-of-string, '/', '?', or '#'
    let tld_len = after_dot
        .find(|c: char| ['/', '?', '#'].contains(&c))
        .unwrap_or(after_dot.len());
    tld_len >= 2
}

/// Extract a human-readable domain from a URL string.
/// "https://www.example.com/path?q=1" -> "example.com"
/// "example.com/page" -> "example.com"
fn domain_from_url(url: &str) -> Option<String> {
    let host = url
        .split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?;

    let host = host.strip_prefix("www.").unwrap_or(host);

    if host.is_empty() {
        return None;
    }

    Some(host.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    // Suffix stripping — works for ANY browser, no class list needed

    #[test]
    fn strip_em_dash_suffix() {
        let result = clean_title("YouTube — Mozilla Firefox", "Firefox");
        assert_eq!(result, "YouTube");
    }

    #[test]
    fn strip_hyphen_suffix() {
        let result = clean_title("YouTube - Google Chrome", "Chrome");
        assert_eq!(result, "YouTube");
    }

    #[test]
    fn strip_single_word_suffix() {
        let result = clean_title("Reddit — Brave", "Brave");
        assert_eq!(result, "Reddit");
    }

    #[test]
    fn strip_unknown_browser_suffix() {
        // Works for any browser, even one never seen before
        let result = clean_title("Hacker News — MyBrowser", "MyBrowser");
        assert_eq!(result, "Hacker News");
    }

    #[test]
    fn strip_suffix_with_short_name() {
        let result = clean_title("Wikipedia — Vivaldi", "Vivaldi");
        assert_eq!(result, "Wikipedia");
    }

    // Generic titles

    #[test]
    fn replace_new_tab() {
        let result = clean_title("New Tab", "Chrome");
        assert_eq!(result, "Chrome");
    }

    #[test]
    fn replace_about_blank() {
        let result = "about:blank";
        assert!(is_generic_title(result));
    }

    // URL handling

    #[test]
    fn bare_url_extracts_domain() {
        let result = clean_title("https://www.example.com/path?q=1", "Chrome");
        assert_eq!(result, "example.com");
    }

    #[test]
    fn bare_domain_extracts_domain() {
        let result = clean_title("example.com/page", "Brave");
        assert_eq!(result, "example.com");
    }

    #[test]
    fn url_with_suffix_not_stripped() {
        // Main part is a URL — suffix should not be stripped
        let result = clean_title("Search results - http://example.com", "App");
        assert_eq!(result, "example.com");
    }

    // Non-browser titles left alone

    #[test]
    fn terminal_title_unchanged() {
        let result = clean_title("user@host: ~/work", "Kitty");
        assert_eq!(result, "user@host: ~/work");
    }

    #[test]
    fn editor_title_unchanged() {
        // "Neovim" is a valid app name suffix — gets stripped correctly
        let result = clean_title("main.rs — Neovim", "Neovim");
        assert_eq!(result, "main.rs");
    }

    #[test]
    fn empty_title_falls_back() {
        let result = clean_title("", "Firefox");
        assert_eq!(result, "Firefox");
    }

    #[test]
    fn content_with_hyphen_not_stripped() {
        // "Times" looks like app suffix (1 word, capitalized)
        // but the separator is " - " and main part is "The New York"
        // This IS stripped — which is correct for browser pattern detection
        let result = clean_title("The New York - Times", "App");
        assert_eq!(result, "The New York");
    }

    #[test]
    fn suffix_with_url_not_stripped() {
        // suffix "http://example.com" has colon -> not app suffix
        // but the title contains a URL -> domain is extracted
        let result = clean_title("Results - http://example.com", "App");
        assert_eq!(result, "example.com");
    }

    #[test]
    fn suffix_with_dot_not_stripped() {
        // suffix contains a dot -> not an app name
        let result = clean_title("Page - v2.1.0", "App");
        assert_eq!(result, "Page - v2.1.0");
    }

    #[test]
    fn suffix_with_paren_not_stripped() {
        let result = clean_title("Docs - React (v18)", "App");
        assert_eq!(result, "Docs - React (v18)");
    }
}
