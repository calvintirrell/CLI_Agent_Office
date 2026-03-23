#!/bin/bash
# Launch CLI Agent Office (Tauri) if not already running.
# Called by Claude Code's SessionStart hook.

APP_NAME="CLI Agent Office"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# macOS: check for .app bundle first, then fall back to cargo binary
if [ "$(uname)" = "Darwin" ]; then
  APP_PATH="$PROJECT_DIR/src-tauri/target/release/bundle/macos/$APP_NAME.app"
  if [ -d "$APP_PATH" ] && ! pgrep -f "$APP_NAME" > /dev/null 2>&1; then
    open "$APP_PATH" &
  fi
else
  # Linux/Windows: run the binary directly
  BIN_PATH="$PROJECT_DIR/src-tauri/target/release/cli-agent-office"
  if [ -x "$BIN_PATH" ] && ! pgrep -f "$APP_NAME" > /dev/null 2>&1; then
    "$BIN_PATH" &
  fi
fi

exit 0
