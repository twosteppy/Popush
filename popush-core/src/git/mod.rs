//! Local git subsystem. The pure part, classifying and converting remote
//! URLs, lives in [`remote`] and is tested here.
//! The `git2` operations (status, stage, commit, push with agent credentials,
//! the `notify` watcher) live in the binary layer against a real repository.

pub mod remote;

pub use remote::{classify_remote, https_to_ssh, RemoteKind};
