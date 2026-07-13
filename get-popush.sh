#!/usr/bin/env bash
#
# Popush one-command installer.
#
#   curl -fsSL https://raw.githubusercontent.com/twosteppy/popush/main/get-popush.sh | bash
#
# Downloads the latest prebuilt Popush AppImage from GitHub Releases (no cloning,
# no compiling), verifies its checksum, drops it into your app launcher, and puts
# a double-clickable icon on your Desktop. Nothing else. It never touches your
# ~/.ssh or your servers.
#
# Environment overrides (optional):
#   POPUSH_REPO=owner/name   Use a different GitHub repo (default twosteppy/popush).
#   POPUSH_VERSION=v1.2.3    Install a specific tag instead of the latest.

set -euo pipefail

REPO="${POPUSH_REPO:-twosteppy/popush}"
APP_ID="dev.popush.Popush"
RAW="https://raw.githubusercontent.com/${REPO}/main"

# ---------------------------------------------------------------------------
# Pretty output
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  BOLD=$'\e[1m'; DIM=$'\e[2m'; RESET=$'\e[0m'
  PINK=$'\e[38;5;211m'; GREEN=$'\e[38;5;71m'; RED=$'\e[38;5;167m'; AMBER=$'\e[38;5;179m'
else
  BOLD=""; DIM=""; RESET=""; PINK=""; GREEN=""; RED=""; AMBER=""
fi
step()  { printf '%b> %s%b\n' "$PINK$BOLD" "$1" "$RESET"; }
ok()    { printf '%b  ok %s%b\n' "$GREEN" "$1" "$RESET"; }
info()  { printf '%b  %s%b\n' "$DIM" "$1" "$RESET"; }
warn()  { printf '%b  ! %s%b\n' "$AMBER" "$1" "$RESET"; }
die()   { printf '%b%s%b\n' "$RED$BOLD" "$1" "$RESET" >&2; exit 1; }
have()  { command -v "$1" >/dev/null 2>&1; }

printf '\n%b%sPopush%b  Your VPS, one click away.\n\n' "$PINK$BOLD" "" "$RESET"

# ---------------------------------------------------------------------------
# 0. Sanity checks
# ---------------------------------------------------------------------------
[[ "$(uname -s)" == "Linux" ]] || die "Popush is Linux only."
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" || "$ARCH" == "amd64" ]] || \
  die "The prebuilt AppImage is x86_64 only (you are on ${ARCH}). Build from source: https://github.com/${REPO}"
have curl || die "curl is required. Install it (e.g. 'sudo dnf install curl') and re-run."

DL="curl -fsSL"

# ---------------------------------------------------------------------------
# 1. Find the release and the AppImage asset
# ---------------------------------------------------------------------------
if [[ -n "${POPUSH_VERSION:-}" ]]; then
  API="https://api.github.com/repos/${REPO}/releases/tags/${POPUSH_VERSION}"
  step "Looking up Popush ${POPUSH_VERSION}"
else
  API="https://api.github.com/repos/${REPO}/releases/latest"
  step "Looking up the latest Popush release"
fi

RELEASE_JSON="$($DL "$API" 2>/dev/null || true)"
[[ -n "$RELEASE_JSON" ]] || die "Could not reach GitHub. Check your connection and try again."

# Pull the AppImage download URL out of the release JSON without needing jq.
APPIMAGE_URL="$(printf '%s' "$RELEASE_JSON" \
  | grep -oE '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*\.AppImage"' \
  | head -1 | grep -oE 'https[^"]*\.AppImage' || true)"

if [[ -z "$APPIMAGE_URL" ]]; then
  if printf '%s' "$RELEASE_JSON" | grep -q '"message".*"Not Found"'; then
    die "No Popush release found yet for ${REPO}. Ask the maintainer to publish one, or build from source: https://github.com/${REPO}"
  fi
  die "This release has no AppImage attached. Build from source instead: https://github.com/${REPO}"
fi
SUMS_URL="$(printf '%s' "$RELEASE_JSON" \
  | grep -oE 'https[^"]*SHA256SUMS[^"]*' | head -1 || true)"
ok "Found $(basename "$APPIMAGE_URL")"

# ---------------------------------------------------------------------------
# 2. Download and verify
# ---------------------------------------------------------------------------
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
APP_TMP="$TMP/popush.AppImage"

step "Downloading Popush"
$DL "$APPIMAGE_URL" -o "$APP_TMP" || die "Download failed."
ok "Downloaded ($(du -h "$APP_TMP" | cut -f1))"

if [[ -n "$SUMS_URL" ]] && have sha256sum; then
  step "Verifying checksum"
  SUMS="$($DL "$SUMS_URL" 2>/dev/null || true)"
  WANT="$(printf '%s' "$SUMS" | grep -F "$(basename "$APPIMAGE_URL")" | awk '{print $1}' | head -1)"
  if [[ -n "$WANT" ]]; then
    GOT="$(sha256sum "$APP_TMP" | awk '{print $1}')"
    [[ "$WANT" == "$GOT" ]] || die "Checksum mismatch. Refusing to install a tampered download."
    ok "Checksum verified"
  else
    warn "No checksum line for this file; skipping verification."
  fi
else
  warn "No published checksums; skipping verification."
fi

# ---------------------------------------------------------------------------
# 3. Install the AppImage to a stable location
# ---------------------------------------------------------------------------
BIN_HOME="$HOME/.local/bin"
mkdir -p "$BIN_HOME"
LAUNCH_TARGET="$BIN_HOME/popush.AppImage"
install -m 0755 "$APP_TMP" "$LAUNCH_TARGET"
ok "Installed to $LAUNCH_TARGET"

# ---------------------------------------------------------------------------
# 4. Icon + launcher entry + Desktop shortcut
# ---------------------------------------------------------------------------
step "Adding Popush to your applications and desktop"

for size in 32 128 256 512; do
  dst_dir="$HOME/.local/share/icons/hicolor/${size}x${size}/apps"
  mkdir -p "$dst_dir"
  $DL "${RAW}/packaging/desktop/icons/${APP_ID}-${size}.png" -o "$dst_dir/${APP_ID}.png" 2>/dev/null || true
done
svg_dir="$HOME/.local/share/icons/hicolor/scalable/apps"
mkdir -p "$svg_dir"
$DL "${RAW}/packaging/desktop/icons/${APP_ID}.svg" -o "$svg_dir/${APP_ID}.svg" 2>/dev/null || true
ok "Installed icons"

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
DESKTOP_FILE="$DESKTOP_DIR/${APP_ID}.desktop"
cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=Popush
GenericName=VPS deployment client
Comment=Your VPS, one click away.
Exec=$LAUNCH_TARGET
Icon=$APP_ID
Terminal=false
Categories=Development;Utility;
Keywords=deploy;ssh;vps;git;server;
StartupWMClass=Popush
EOF
chmod 0644 "$DESKTOP_FILE"
ok "Installed launcher entry"

have update-desktop-database && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
have gtk-update-icon-cache && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

DESKTOP_HOME="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
if [[ -d "$DESKTOP_HOME" ]]; then
  install -m 0755 "$DESKTOP_FILE" "$DESKTOP_HOME/${APP_ID}.desktop"
  have gio && gio set "$DESKTOP_HOME/${APP_ID}.desktop" metadata::trusted true >/dev/null 2>&1 || true
  ok "Placed a Popush shortcut on your Desktop"
fi

# ---------------------------------------------------------------------------
# 5. Friendly finish
# ---------------------------------------------------------------------------
# AppImages need FUSE to run. Most desktops have it; warn kindly if not.
if ! ldconfig -p 2>/dev/null | grep -q 'libfuse\.so\.2' && ! have fusermount && ! have fusermount3; then
  warn "AppImages need FUSE. If Popush will not start, install it:"
  info "Fedora:  sudo dnf install fuse"
  info "Ubuntu:  sudo apt install libfuse2"
fi

printf '\n%bDone.%b Popush is in your app launcher, search for "Popush".\n' "$GREEN$BOLD" "$RESET"
printf '%bYou can also double-click the Popush icon on your Desktop.%b\n\n' "$DIM" "$RESET"
info "First run: if you do not have an SSH key yet, run 'ssh-keygen -t ed25519',"
info "then open Popush and click \"Add your first server\". No config files, no terminal."
