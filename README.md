# `DDS` 

![Rust](https://img.shields.io/badge/Rust-1.85+-orange?style=for-the-badge\&logo=rust)
![License MIT](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
![Platform: Linux](https://img.shields.io/badge/Platform-Linux-blue?style=for-the-badge) 
![WM](https://img.shields.io/badge/WM-Hyprland-purple?style=for-the-badge)

**Dynamic Discord Rich Presence for Hyprland (Wayland).**
Automatically updates your Discord status based on the active window.

---

## ✨ Features

* **Dynamic status** — builds your Discord Rich Presence live from the active window (class → name, title → details). No need to list every app.
* **Browser support** — automatically cleans browser tab titles (strips `- Mozilla Firefox` style suffixes, extracts domains from bare URLs, replaces "New Tab" with the app name).
* Any application works out of the box; only minimal optional config
* Lightweight and fast (Rust + Hyprland events)
* Works on Hyprland (Wayland)

---

## 🏗 Installation and Run

### Quick install (recommended)

```bash
git clone https://github.com/Veridian-Zenith/DDS.git
cd DDS
./scripts/install.sh --release
systemctl --user daemon-reload
systemctl --user enable --now discord-monitor.service
```

This installs the binary and a systemd user service that auto-launches the status
app whenever Discord is running, and stops it when Discord closes. It restarts
automatically if Discord is reopened.

### Manual run (no service)

```bash
cargo run --release
```

> Or use `yay -S ddsh-bin` / `yay -S ddsh-git`

### 2. Configure `config.json` (in the `~/.local/share/dynamic-drpc-hyprland`)

The status is generated dynamically from the active window, so the config is
minimal. A default one is created automatically on first run:

```json
{
  "app_id": "1460605258072985705",
  "default_large_image": "arch",
  "details_from_title": true,
  "image_map": {
    "kitty": "kitty",
    "org.mozilla.firefox": "firefox"
  },
  "name_map": {
    "org.mozilla.firefox": "Firefox"
  }
}
```

How it works:

* `app_id` — your Discord Application ID (from the Developer Portal, **not a bot token**).
* `details_from_title` — (default `true`) use the live window title as the details line.
* The **state** is always `"Using <name>"` where `<name>` is the window's class,
  auto-prettyfied (e.g. `org.mozilla.firefox` → `Firefox`, `kitty` → `Kitty`).
* `image_map` — optional: map a window class to a Discord asset key for the
  large image. Anything unlisted falls back to `default_large_image`.
* `name_map` — optional: override the auto-prettyfied display name for a class.

You only add entries to `image_map` / `name_map` for apps you want to brand
nicely — everything else just works with the raw class name.

> **Note:** image asset keys still have to exist in your Discord app's
> Art Assets (Developer Portal), since assets can't be uploaded automatically.

---

## 🛠 Troubleshooting

* Discord must be **online** and **not in Invisible** mode
* Any asset keys used in `image_map` / `default_large_image` must exist in Discord Developer Portal → Art Assets
* Of course, you should have an internet connection :)

---

## 📝 License

MIT License — see [LICENSE](LICENSE)
