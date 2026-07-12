//! Configuration (§7): human-editable TOML on disk, the source of truth Popush
//! shares with the user (D6). No file Popush writes holds a secret (D7): keys are
//! referenced by path, tokens live in the keyring, passphrases are never stored.

pub mod loader;
pub mod schema;
pub mod writer;

pub use loader::{load_from_str, validate};
pub use schema::{
    ChangeKind, ChangedFile, Config, GitStatus, Preferences, ServerConfig, ServiceConfig,
    SiteConfig, SiteStatus, Theme,
};
pub use writer::{remove_server, remove_site, to_toml, upsert_server};
