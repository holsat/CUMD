#!/usr/bin/env bash
# Script to dynamically manage firewall ports for desktop-mcp-daemon
set -e

ACTION="${1:-open}"
PORT="${2:-8443}"

if command -v ufw >/dev/null 2>&1 && ufw status | grep -qw "active"; then
    if [ "$ACTION" = "open" ]; then
        echo "Opening port $PORT/tcp via ufw..."
        ufw allow "$PORT/tcp"
    else
        echo "Closing port $PORT/tcp via ufw..."
        ufw delete allow "$PORT/tcp" || true
    fi
elif command -v firewall-cmd >/dev/null 2>&1 && firewall-cmd --state >/dev/null 2>&1; then
    if [ "$ACTION" = "open" ]; then
        echo "Opening port $PORT/tcp via firewalld..."
        firewall-cmd --add-port="$PORT/tcp"
    else
        echo "Closing port $PORT/tcp via firewalld..."
        firewall-cmd --remove-port="$PORT/tcp" || true
    fi
else
    echo "No active ufw or firewalld detected; skipping firewall rules."
fi
