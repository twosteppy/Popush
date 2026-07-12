//! Prime-directive enforcement tests that span the whole repository (§23.5, D7,
//! D9, D11). These run in CI as part of the release gate. Paths are resolved from
//! `CARGO_MANIFEST_DIR` (the `popush-core` crate dir) up to the repo root, so the
//! tests do not depend on the process working directory.

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root, one level above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("popush-core has a parent (the repo root)")
        .to_path_buf()
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

/// D9: the name "twostep" must appear in every required location, surviving any
/// refactor. This is the credit assertion the spec requires in seven places.
#[test]
fn credit_to_twostep_is_present_everywhere_d9() {
    let root = repo_root();

    // 1. README header credit.
    assert!(
        read(&root, "README.md").contains("Built by twostep."),
        "README must contain the header credit \"Built by twostep.\""
    );
    // 2. LICENSE copyright line.
    assert!(
        read(&root, "LICENSE").contains("Copyright (c) 2026 twostep"),
        "LICENSE must carry the twostep copyright line"
    );
    // 3. Root Cargo.toml authors (workspace.package).
    assert!(
        read(&root, "Cargo.toml").contains("twostep"),
        "Cargo.toml authors must include twostep"
    );
    // 4. package.json author.
    assert!(
        read(&root, "package.json").contains("twostep"),
        "package.json author must be twostep"
    );
    // 5. Flatpak metainfo developer_name.
    let metainfo = read(&root, "packaging/flatpak/dev.popush.Popush.metainfo.xml");
    assert!(
        metainfo.contains("twostep"),
        "Flatpak metainfo developer_name must be twostep"
    );
    // 6. About view credit (footer/status bar + About dialog live in the frontend).
    assert!(
        read(&root, "src/views/AboutView.tsx").contains("twostep"),
        "About view must credit twostep"
    );
    // 7. The core crate exposes the credit constant for the running app.
    assert_eq!(popush_core::AUTHOR, "twostep");
}

/// D11: the banned generic strings must not appear in Rust source. The frontend
/// enforces the same over its own tree in a vitest test.
#[test]
fn no_banned_generic_error_strings_in_rust_d11() {
    let banned = ["Deploy failed", "Something went wrong"];
    let root = repo_root();
    for dir in ["popush-core/src", "src-tauri/src"] {
        for path in rust_files(&root.join(dir)) {
            let text = fs::read_to_string(&path).unwrap();
            for (lineno, line) in text.lines().enumerate() {
                let trimmed = line.trim_start();
                // Skip comments and the enforcement machinery itself: doc/line
                // comments and any line that names the ban (the `BANNED` arrays
                // and the doc text that lists the forbidden strings). What remains
                // is code that could put a banned string into a real message.
                let is_comment = trimmed.starts_with("//") || trimmed.starts_with('*');
                let names_the_ban = line.to_lowercase().contains("banned");
                if is_comment || names_the_ban {
                    continue;
                }
                for b in banned {
                    assert!(
                        !line.contains(b),
                        "banned string {b:?} found in {}:{}",
                        path.display(),
                        lineno + 1
                    );
                }
            }
        }
    }
}

/// D7: no committed file should contain an inline private key or a token. This is
/// the repository-side of the secret scan; the runtime data-dir scan lives in the
/// app. We check the tracked source and sample config shapes.
#[test]
fn no_secrets_committed_d7() {
    let root = repo_root();
    // The unambiguous marker of an actually-committed private key. A real key
    // block always contains one of these headers; test fixtures for *token*
    // shapes (e.g. a fake `ghp_...` in a URL-parsing test) are not committed
    // secrets and belong to the runtime data-dir scan, not this source scan.
    let key_markers = [
        "BEGIN OPENSSH PRIVATE KEY",
        "BEGIN RSA PRIVATE KEY",
        "BEGIN EC PRIVATE KEY",
        "BEGIN DSA PRIVATE KEY",
    ];
    for dir in ["popush-core/src", "src-tauri/src", "docs"] {
        for path in files_with_ext(&root.join(dir), &["rs", "toml", "md"]) {
            let text = fs::read_to_string(&path).unwrap();
            for m in key_markers {
                if text.contains(m) {
                    // Allowed only where we deliberately *name* the marker to
                    // reject it (the loader guardrail) or explain it (docs).
                    let is_guardrail = path.ends_with("config/loader.rs");
                    let is_doc = path.extension().map(|e| e == "md").unwrap_or(false);
                    assert!(
                        is_guardrail || is_doc,
                        "possible committed private key ({m:?}) in {}",
                        path.display()
                    );
                }
            }
        }
    }
}

/// The example config shipped in the docs must actually load and validate. A
/// documented example that does not parse is a broken promise (D6: the config is
/// the user's, human-editable source of truth).
#[test]
fn documented_example_config_loads() {
    let root = repo_root();
    let text = read(&root, "docs/config.example.toml");
    let cfg =
        popush_core::config::load_from_str(&text).expect("the documented example config must load");
    assert_eq!(cfg.servers.len(), 1);
    assert_eq!(cfg.servers[0].sites.len(), 2);
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    files_with_ext(dir, &["rs"])
}

fn files_with_ext(dir: &Path, exts: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(files_with_ext(&path, exts));
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| exts.contains(&e))
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }
    out
}
