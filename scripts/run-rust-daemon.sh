#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
IPC_DIR="${NANOCLAW_IPC_DIR:-$ROOT_DIR/data/ipc/main}"
ASSISTANT="${ASSISTANT_NAME:-Andy}"
POLL_MS="${NANOCLAW_POLL_MS:-1000}"

mkdir -p "$IPC_DIR/messages"

cd "$ROOT_DIR/rust"
exec cargo run -p nanoclawd -- --daemon --ipc-dir "$IPC_DIR" --assistant-name "$ASSISTANT" --poll-ms "$POLL_MS"
