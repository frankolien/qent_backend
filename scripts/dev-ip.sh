#!/usr/bin/env bash
# Rewrite mobile/.env so API_BASE_URL points at this machine's current LAN
# IP. Solves the "phone/simulator can't reach my dev server because DHCP
# handed me a new IP overnight" cycle.
#
# Usage:
#   scripts/dev-ip.sh           # auto-detect via Wi-Fi (en0), fall back to en1
#   scripts/dev-ip.sh 10.0.0.5  # force a specific IP (e.g. when on Ethernet)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ENV_FILE="$REPO_ROOT/mobile/.env"
PORT="${PORT:-8080}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "✗ $ENV_FILE not found" >&2
  exit 1
fi

if [[ $# -ge 1 ]]; then
  IP="$1"
else
  # Try common interfaces in order. en0 = Wi-Fi on most Macs, en1 = Ethernet
  # adapter or second Wi-Fi. ipconfig prints nothing (rc=0) if the interface
  # has no IPv4, so we fall through.
  IP="$(ipconfig getifaddr en0 || true)"
  if [[ -z "$IP" ]]; then IP="$(ipconfig getifaddr en1 || true)"; fi
fi

if [[ -z "$IP" ]]; then
  echo "✗ Could not detect a LAN IP. Are you connected to a network?" >&2
  echo "  Hint: pass an IP explicitly: scripts/dev-ip.sh 192.168.1.50" >&2
  exit 1
fi

NEW_URL="http://${IP}:${PORT}/api"

# Idempotent: replace whatever URL is on the API_BASE_URL line, regardless
# of host/port. -i '' is BSD sed (macOS) — no backup file left behind.
sed -i '' -E "s|^API_BASE_URL=.*$|API_BASE_URL=${NEW_URL}|" "$ENV_FILE"

echo "✓ API_BASE_URL → $NEW_URL"
