#!/usr/bin/env bash
# One-shot mobile dev launcher: refresh API_BASE_URL with the current LAN
# IP, then start Flutter. Any args you pass after the script name are
# forwarded to `flutter run` (e.g. `-d <device-id>`).
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
"$REPO_ROOT/scripts/dev-ip.sh"
cd "$REPO_ROOT/mobile"
exec flutter run "$@"
