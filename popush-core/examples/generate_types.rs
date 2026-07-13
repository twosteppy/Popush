use std::fmt::Write as _;

use popush_core::adapters::Capabilities;
use popush_core::command_log::{CommandLogEntry, CommandOutcome};
use popush_core::config::{
    ChangeKind, ChangedFile, Config, GitStatus, Preferences, ServerConfig, ServiceConfig,
    SiteConfig, SiteStatus, Theme,
};
use popush_core::error::{
    AdapterError, AppError, AuthFailureReason, ConfigError, GitError, NextAction, PipelineError,
    SshError, UserMessage,
};
use popush_core::github::{CiStatus, LatestCommit};
use popush_core::ids::{PipelineId, ServerId, SiteId, StreamId};
use popush_core::ssh::HostKeyDecision;
use popush_core::wizard::{Check, CheckStatus, Fix, FixPreview};
use ts_rs::TS;

macro_rules! decl {
    ($out:expr, $($t:ty),+ $(,)?) => {
        $(
            writeln!($out, "export {}\n", <$t as TS>::decl()).unwrap();
        )+
    };
}

fn main() {
    let mut out = String::new();
    out.push_str("// Generated from the Rust types. Do not edit by hand.\n\n");

    decl!(out, ServerId, SiteId, PipelineId, StreamId);
    decl!(
        out,
        Config,
        Preferences,
        Theme,
        ServerConfig,
        SiteConfig,
        ServiceConfig,
        SiteStatus,
        GitStatus,
        ChangedFile,
        ChangeKind
    );
    decl!(out, popush_core::config::schema::ServiceKind);
    decl!(out, Capabilities);
    decl!(out, HostKeyDecision);
    decl!(
        out,
        UserMessage,
        NextAction,
        AppError,
        AuthFailureReason,
        SshError,
        GitError,
        AdapterError,
        ConfigError,
        PipelineError
    );
    decl!(
        out,
        popush_core::pipeline::Step,
        popush_core::pipeline::StepState,
        popush_core::pipeline::StepOutcome,
        popush_core::pipeline::PipelineState,
        popush_core::pipeline::step::StepEntry
    );
    decl!(out, Check, CheckStatus, FixPreview, Fix);
    decl!(out, CommandOutcome, CommandLogEntry);
    decl!(out, LatestCommit, CiStatus);

    let path = std::path::Path::new("src/types/generated.ts");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create src/types");
    }
    std::fs::write(path, out).expect("write generated.ts");
    eprintln!("wrote {}", path.display());
}
