#!/bin/sh
# install.sh — install discord-dynamic-status binaries and auto-launch services.
#
# Usage:
#   ./scripts/install.sh --hyprland [--release]   Install ddsh (Hyprland)
#   ./scripts/install.sh --cosmic   [--release]   Install ddsc (COSMIC)
#   ./scripts/install.sh --all      [--release]   Install both
#
# If no desktop flag is given, installs both.

set -e

BINDIR="${HOME}/.local/bin"
SERVICEDIR="${HOME}/.config/systemd/user"
AUTOSTARTDIR="${HOME}/.config/autostart"

mkdir -p "$BINDIR" "$SERVICEDIR" "$AUTOSTARTDIR"

RELEASE=""
DESKTOP=""

for arg in "$@"; do
    case "$arg" in
        --release)  RELEASE="--release" ;;
        --hyprland) DESKTOP="hyprland" ;;
        --cosmic)   DESKTOP="cosmic" ;;
        --all)      DESKTOP="all" ;;
        *)          echo "Unknown option: $arg"; exit 1 ;;
    esac
done

# Default to both if no desktop specified
if [ -z "$DESKTOP" ]; then
    DESKTOP="all"
fi

build_binary() {
    local name="$1"
    if [ "$RELEASE" = "--release" ]; then
        echo "Building $name (release)..."
        cargo build --release -p "$name"
        cp "target/release/$name" "$BINDIR/$name"
    else
        echo "Building $name (debug)..."
        cargo build -p "$name"
        cp "target/debug/$name" "$BINDIR/$name"
    fi
}

install_hyprland() {
    build_binary ddsh

    cp scripts/discord-monitor.sh "$BINDIR/discord-monitor.sh"
    chmod +x "$BINDIR/discord-monitor.sh"

    cp scripts/discord-monitor-hyprland.service "$SERVICEDIR/discord-monitor.service"

    echo ""
    echo "Installed (Hyprland):"
    echo "  $BINDIR/ddsh"
    echo "  $BINDIR/discord-monitor.sh"
    echo "  $SERVICEDIR/discord-monitor.service"
    echo ""
    echo "To enable (starts automatically on login):"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable --now discord-monitor.service"
    echo ""
    echo "To start immediately:"
    echo "  systemctl --user start discord-monitor.service"
}

install_cosmic() {
    build_binary ddsc

    cp scripts/discord-monitor.sh "$BINDIR/discord-monitor.sh"
    chmod +x "$BINDIR/discord-monitor.sh"

    cp scripts/discord-monitor-cosmic.service "$SERVICEDIR/discord-monitor.service"

    # Install .desktop file for COSMIC autostart
    mkdir -p "$AUTOSTARTDIR"
    cp cosmic/autostart/ddsc.desktop "$AUTOSTARTDIR/ddsc.desktop"

    echo ""
    echo "Installed (COSMIC):"
    echo "  $BINDIR/ddsc"
    echo "  $BINDIR/discord-monitor.sh"
    echo "  $SERVICEDIR/discord-monitor.service"
    echo "  $AUTOSTARTDIR/ddsc.desktop"
    echo ""
    echo "To enable via systemd (starts automatically on login):"
    echo "  systemctl --user daemon-reload"
    echo "  systemctl --user enable --now discord-monitor.service"
    echo ""
    echo "Or enable via COSMIC autostart settings (ddsc.desktop is installed)."
    echo ""
    echo "To start immediately:"
    echo "  systemctl --user start discord-monitor.service"
}

case "$DESKTOP" in
    hyprland) install_hyprland ;;
    cosmic)   install_cosmic ;;
    all)
        install_hyprland
        echo "---"
        install_cosmic
        ;;
esac
