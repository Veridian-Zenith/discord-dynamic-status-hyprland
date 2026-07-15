#!/bin/sh
# install.sh — install ddsh and the auto-launch service.
#
# Usage: ./scripts/install.sh [--release]

set -e

BINDIR="${HOME}/.local/bin"
SERVICEDIR="${HOME}/.config/systemd/user"

mkdir -p "$BINDIR" "$SERVICEDIR"

# Build
if [ "$1" = "--release" ]; then
    echo "Building release..."
    cargo build --release
    cp target/release/ddsh "$BINDIR/ddsh"
else
    echo "Building debug..."
    cargo build
    cp target/debug/ddsh "$BINDIR/ddsh"
fi

# Monitor script
cp scripts/discord-monitor.sh "$BINDIR/discord-monitor.sh"
chmod +x "$BINDIR/discord-monitor.sh"

# Systemd service
cp scripts/discord-monitor.service "$SERVICEDIR/discord-monitor.service"

echo ""
echo "Installed:"
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
