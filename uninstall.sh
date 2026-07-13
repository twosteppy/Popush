#!/usr/bin/env bash
#
# Remove Popush's desktop integration and the installed AppImage. Built by twostep.
# This does NOT touch your config (~/.config/popush) or your servers, delete that
# directory by hand if you want to remove your settings too.
set -euo pipefail

APP_ID="dev.popush.Popush"
GREEN=$'\e[38;5;71m'; DIM=$'\e[2m'; RESET=$'\e[0m'
say() { printf '%b  ✓ %s%b\n' "$GREEN" "$1" "$RESET"; }

rm -f "$HOME/.local/bin/popush.AppImage" && say "Removed installed AppImage" || true
rm -f "$HOME/.local/share/applications/${APP_ID}.desktop" && say "Removed launcher entry" || true

DESKTOP_HOME="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
rm -f "$DESKTOP_HOME/${APP_ID}.desktop" && say "Removed Desktop shortcut" || true

for size in 16 24 32 48 64 128 256 512 scalable; do
  ext=png; [[ "$size" == scalable ]] && ext=svg
  rm -f "$HOME/.local/share/icons/hicolor/${size}x${size}/apps/${APP_ID}.png" 2>/dev/null || true
  rm -f "$HOME/.local/share/icons/hicolor/${size}/apps/${APP_ID}.${ext}" 2>/dev/null || true
done
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/${APP_ID}.svg" 2>/dev/null || true
say "Removed icons"

command -v update-desktop-database >/dev/null && update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

printf '\nPopush desktop integration removed.\n%bYour config at ~/.config/popush was left untouched.%b\n' "$DIM" "$RESET"
