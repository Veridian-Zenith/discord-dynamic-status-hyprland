#!/bin/sh
# discord-monitor.sh — keep discord-dynamic-status alive while Discord runs.
#
# Usage: discord-monitor.sh <binary-name>
#   e.g. discord-monitor.sh ddsh    (for Hyprland)
#        discord-monitor.sh ddsc    (for COSMIC)
#
# Designed to be run as a systemd user service (or exec-once).
# Polls for Discord's process every POLL_INTERVAL seconds and starts/stops the
# status app accordingly.

BINARY="${1:-ddsh}"
POLL_INTERVAL=3
PID_FILE="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/drpc-${BINARY}.pid"

cleanup() {
    [ -f "$PID_FILE" ] && kill "$(cat "$PID_FILE")" 2>/dev/null && rm -f "$PID_FILE"
    exit 0
}
trap cleanup TERM INT HUP

# Detect the Discord process name (works for stable, PTB, canary, etc.)
discord_running() {
    pgrep -x Discord >/dev/null 2>&1 || pgrep -x discord >/dev/null 2>&1
}

app_running() {
    [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null
}

start_app() {
    $BINARY &
    echo $! > "$PID_FILE"
}

stop_app() {
    [ -f "$PID_FILE" ] || return
    pid=$(cat "$PID_FILE")
    kill "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
    rm -f "$PID_FILE"
}

while :; do
    if discord_running; then
        app_running || start_app
    else
        # Discord closed — stop the app so it cleanly resets on next launch
        app_running && stop_app
    fi
    sleep "$POLL_INTERVAL"
done
