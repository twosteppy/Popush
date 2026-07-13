#!/usr/bin/env bash
#
# Popush installer. Built by twostep.
#
# One command to go from a fresh clone to a Popush icon in your application
# launcher and on your desktop:
#
#     bash install.sh
#
# It installs the build prerequisites (on Fedora), builds Popush, and integrates
# it into your desktop, icon, launcher entry, and a double-clickable shortcut on
# your Desktop. It never touches your ~/.ssh or your servers; it only builds and
# installs the app itself.
#
# Flags:
#   --no-deps     Skip installing system packages (assume they are present).
#   --no-desktop  Build only; do not create launcher/desktop entries.
#   --help        Show this help.

set -euo pipefail

# ---------------------------------------------------------------------------
# Pretty output
# ---------------------------------------------------------------------------
if [[ -t 1 ]]; then
  BOLD=$'\e[1m'; DIM=$'\e[2m'; RESET=$'\e[0m'
  VIOLET=$'\e[38;5;99m'; GREEN=$'\e[38;5;71m'; RED=$'\e[38;5;167m'; AMBER=$'\e[38;5;179m'
else
  BOLD=""; DIM=""; RESET=""; VIOLET=""; GREEN=""; RED=""; AMBER=""
fi

step()  { printf '%b▸ %s%b\n' "$VIOLET$BOLD" "$1" "$RESET"; }
ok()    { printf '%b  ✓ %s%b\n' "$GREEN" "$1" "$RESET"; }
info()  { printf '%b  %s%b\n' "$DIM" "$1" "$RESET"; }
warn()  { printf '%b  ! %s%b\n' "$AMBER" "$1" "$RESET"; }
die()   { printf '%b✗ %s%b\n' "$RED$BOLD" "$1" "$RESET" >&2; exit 1; }

APP_ID="dev.popush.Popush"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DO_DEPS=1
DO_DESKTOP=1

for arg in "$@"; do
  case "$arg" in
    --no-deps)    DO_DEPS=0 ;;
    --no-desktop) DO_DESKTOP=0 ;;
    --help|-h)
      cat <<'HELP'
Popush installer, one command from a fresh clone to a launchable Popush icon.

  bash install.sh

Installs build prerequisites (on Fedora), builds Popush, and adds it to your
application launcher and Desktop. It never touches ~/.ssh or your servers.

Flags:
  --no-deps     Skip installing system packages (assume they are present).
  --no-desktop  Build only; do not create launcher/desktop entries.
  --help        Show this help.
HELP
      exit 0 ;;
    *) die "Unknown option: $arg (try --help)" ;;
  esac
done

printf '\n%b%sPopush%b, Your VPS, one click away.\n%bBuilt by twostep.%b\n\n' \
  "$VIOLET$BOLD" "" "$RESET" "$DIM" "$RESET"

# ---------------------------------------------------------------------------
# 1. Prerequisites
# ---------------------------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }

if [[ "$DO_DEPS" -eq 1 ]]; then
  step "Checking build prerequisites"
  if have dnf; then
    PKGS=(webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel curl git)
    MISSING=()
    for p in "${PKGS[@]}"; do rpm -q "$p" >/dev/null 2>&1 || MISSING+=("$p"); done
    if [[ ${#MISSING[@]} -gt 0 ]]; then
      info "Installing: ${MISSING[*]}"
      sudo dnf install -y "${MISSING[@]}" || die "Package install failed. Install them manually and re-run with --no-deps."
    fi
    ok "System packages present"
  else
    warn "Not a dnf/Fedora system. Ensure WebKitGTK 4.1, GTK3, librsvg, and OpenSSL dev headers are installed, then re-run with --no-deps."
  fi

  if ! have cargo; then
    info "Installing the Rust toolchain via rustup"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  have cargo || die "cargo is not on PATH. Open a new shell (or 'source ~/.cargo/env') and re-run."
  ok "Rust toolchain present ($(cargo --version))"

  if ! have pnpm; then
    if have corepack; then
      corepack enable pnpm || true
    fi
    have pnpm || { info "Installing pnpm globally via npm"; have npm || die "Node.js/npm not found. Install nodejs, then re-run."; sudo npm install -g pnpm; }
  fi
  ok "pnpm present ($(pnpm --version))"
fi

# ---------------------------------------------------------------------------
# 2. Build
# ---------------------------------------------------------------------------
cd "$REPO_ROOT"

step "Installing frontend dependencies"
# --ignore-scripts: esbuild's optional postinstall isn't needed (its platform
# binary ships prebuilt) and strict pnpm policies treat a skipped build as fatal.
pnpm install --ignore-scripts || die "pnpm install failed."
ok "Dependencies installed"

step "Generating typed IPC bindings"
cargo run -q -p popush-core --example generate_types >/dev/null || die "Type generation failed."
ok "Types generated"

step "Building Popush (this takes a few minutes the first time)"
pnpm tauri build || die "Build failed. If it is a Rust error, copy it and open an issue."
ok "Build complete"

# ---------------------------------------------------------------------------
# 3. Locate the built artifacts
# ---------------------------------------------------------------------------
BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle"
APPIMAGE="$(find "$BUNDLE_DIR/appimage" -maxdepth 1 -name '*.AppImage' 2>/dev/null | head -1 || true)"
BIN="$REPO_ROOT/src-tauri/target/release/popush"
RPM="$(find "$BUNDLE_DIR/rpm" -maxdepth 1 -name '*.rpm' 2>/dev/null | head -1 || true)"

printf '\n'
step "Build artifacts"
[[ -n "$APPIMAGE" ]] && ok "AppImage: $APPIMAGE"
[[ -n "$RPM" ]]      && ok "RPM:      $RPM"
[[ -f "$BIN" ]]      && ok "Binary:   $BIN"

if [[ "$DO_DESKTOP" -eq 0 ]]; then
  printf '\n%bDone.%b Run it with: %s\n' "$GREEN$BOLD" "$RESET" "${APPIMAGE:-$BIN}"
  exit 0
fi

# ---------------------------------------------------------------------------
# 4. Desktop integration, icon + launcher + Desktop shortcut
# ---------------------------------------------------------------------------
step "Adding Popush to your applications and desktop"

# Choose what the launcher runs: the AppImage if we have one, else the binary.
LAUNCH_TARGET="${APPIMAGE:-$BIN}"
[[ -n "$LAUNCH_TARGET" && -e "$LAUNCH_TARGET" ]] || die "No runnable artifact was produced."

# Keep a stable home for the AppImage so the launcher path does not change.
APPS_HOME="$HOME/.local/bin"
mkdir -p "$APPS_HOME"
if [[ -n "$APPIMAGE" ]]; then
  install -m 0755 "$APPIMAGE" "$APPS_HOME/popush.AppImage"
  LAUNCH_TARGET="$APPS_HOME/popush.AppImage"
  ok "Installed AppImage to $LAUNCH_TARGET"
fi

# Icons into the hicolor theme.
ICON_SRC="$REPO_ROOT/packaging/desktop/icons"
for size in 16 24 32 48 64 128 256 512; do
  src="$ICON_SRC/${APP_ID}-${size}.png"
  [[ -f "$src" ]] || continue
  dst_dir="$HOME/.local/share/icons/hicolor/${size}x${size}/apps"
  mkdir -p "$dst_dir"
  install -m 0644 "$src" "$dst_dir/${APP_ID}.png"
done
# Scalable SVG too.
if [[ -f "$ICON_SRC/${APP_ID}.svg" ]]; then
  dst_dir="$HOME/.local/share/icons/hicolor/scalable/apps"
  mkdir -p "$dst_dir"
  install -m 0644 "$ICON_SRC/${APP_ID}.svg" "$dst_dir/${APP_ID}.svg"
fi
ok "Installed icons"

# The .desktop launcher.
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

# Refresh caches so it appears immediately.
have update-desktop-database && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
have gtk-update-icon-cache && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" >/dev/null 2>&1 || true

# A double-clickable shortcut on the Desktop (KDE/Freedesktop).
DESKTOP_HOME="$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")"
if [[ -d "$DESKTOP_HOME" ]]; then
  install -m 0755 "$DESKTOP_FILE" "$DESKTOP_HOME/${APP_ID}.desktop"
  # KDE Plasma shows the icon (not a script warning) once the file is trusted.
  if have kioclient5; then kioclient5 exec "$DESKTOP_HOME/${APP_ID}.desktop" >/dev/null 2>&1 || true; fi
  if have gio; then gio set "$DESKTOP_HOME/${APP_ID}.desktop" metadata::trusted true >/dev/null 2>&1 || true; fi
  ok "Placed a Popush shortcut on your Desktop"
else
  info "No Desktop folder found; skipped the desktop shortcut (it is still in your app launcher)."
fi

printf '\n%bDone.%b Popush is in your application launcher, search for “Popush”.\n' "$GREEN$BOLD" "$RESET"
printf '%bYou can also double-click the Popush icon on your Desktop.%b\n\n' "$DIM" "$RESET"
info "First run: create an SSH key if you do not have one (ssh-keygen -t ed25519), then click “Add your first server”."
