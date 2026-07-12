//! Integration tests against the containerised test VPS (§23.3).
//!
//! These are `#[ignore]` by default: they require the test VPS from `test-vps/`
//! to be running and an `ssh-agent` holding the throwaway key. They never touch
//! the developer's real environment (Agent Rule 6), all state lives in the
//! container, reached only through env-configured coordinates.
//!
//! Run them with the harness up:
//!
//! ```text
//! cd test-vps && ./generate-key.sh && docker compose up -d --build
//! eval "$(ssh-agent -s)" && ssh-add test-vps/test_key
//! POPUSH_TEST_VPS=localhost:2222 cargo test -p popush --test integration -- --ignored
//! ```
//!
//! The escaping, parsing, and message contracts are already proven by the
//! `popush-core` unit tests; these prove the *wiring* end to end against real
//! `sshd`, `docker`, `pm2`, and `nginx`.

use std::env;

use popush_core::config::ServerConfig;
use popush_core::ssh::{parse_known_hosts, RemoteCommand};
use popush_lib::ssh::SshPool;

/// Read the test VPS coordinates from the environment, or `None` to skip.
fn test_server() -> Option<(ServerConfig, Vec<popush_core::ssh::KnownHost>)> {
    let target = env::var("POPUSH_TEST_VPS").ok()?; // e.g. "localhost:2222"
    let (host, port) = target.split_once(':').unwrap_or((&target, "22"));
    let known = env::var("POPUSH_TEST_KNOWN_HOSTS")
        .ok()
        .map(|s| parse_known_hosts(&s))
        .unwrap_or_default();
    let server = ServerConfig {
        id: popush_core::ids::ServerId("test-vps".into()),
        label: "Test VPS".into(),
        host: host.to_string(),
        port: port.parse().unwrap_or(22),
        username: env::var("POPUSH_TEST_USER").unwrap_or_else(|_| "deploy".into()),
        identity_file: "~/.ssh/id_ed25519".into(),
        proxy_jump: None,
        sites: Vec::new(),
    };
    Some((server, known))
}

#[tokio::test]
#[ignore = "requires the test VPS from test-vps/ and an ssh-agent (§23.3)"]
async fn connects_and_runs_a_command() {
    let Some((server, known)) = test_server() else {
        eprintln!("POPUSH_TEST_VPS not set; skipping");
        return;
    };
    let pool = SshPool::connect(server, known)
        .await
        .expect("should connect to the test VPS");
    let out = pool
        .exec(RemoteCommand::literal("echo popush-ok"))
        .await
        .expect("command should run");
    assert_eq!(out.exit_code, 0);
    assert_eq!(out.stdout.trim(), "popush-ok");
}

#[tokio::test]
#[ignore = "requires the test VPS (§23.3)"]
async fn escaping_holds_against_a_real_shell() {
    // The corpus is proven inert by unit tests; here we prove a real `sh` on the
    // server treats a dangerous argument as one literal word rather than syntax.
    let Some((server, known)) = test_server() else {
        eprintln!("POPUSH_TEST_VPS not set; skipping");
        return;
    };
    let pool = SshPool::connect(server, known).await.expect("connect");
    // If escaping failed, the `; echo pwned` would run as a second command.
    let payload = "value; echo pwned";
    let out = pool
        .exec(RemoteCommand::new("echo {}", vec![payload.to_string()]))
        .await
        .expect("run");
    assert_eq!(out.stdout.trim(), payload, "argument must survive verbatim");
    assert!(!out.stdout.contains("pwned"), "no injected command ran");
}

#[tokio::test]
#[ignore = "requires the test VPS with the docker sample site (§23.3)"]
async fn docker_adapter_reports_status() {
    let Some((server, known)) = test_server() else {
        eprintln!("POPUSH_TEST_VPS not set; skipping");
        return;
    };
    let pool = SshPool::connect(server, known).await.expect("connect");
    let out = pool
        .exec(popush_core::adapters::docker::status_command(
            "/srv/docker-site",
        ))
        .await
        .expect("run status");
    // The parser is unit-tested; here we assert real compose output parses at all.
    let status = popush_core::adapters::docker::parse_status(&out.stdout);
    assert!(
        status.is_ok(),
        "real compose output should parse: {status:?}"
    );
}
