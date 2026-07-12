//! Optional GitHub integration (Phase 10). See [`api`].

pub mod api;

pub use api::{clear_token, get_token, store_token, GitHubClient};
