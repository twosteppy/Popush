// Generated from the Rust types. Do not edit by hand.

export type ServerId = string;

export type SiteId = string;

export type PipelineId = string;

export type StreamId = string;

export type Config = { schema_version: number, preferences: Preferences, server: Array<ServerConfig>, };

export type Preferences = { theme: Theme, accent: string, poll_interval_seconds: bigint, confirm_destructive: boolean, default_branch: string, };

export type Theme = "system" | "dark" | "light";

export type ServerConfig = { id: ServerId, label: string, host: string, port: number, username: string, identity_file: string, proxy_jump: string | null, site: Array<SiteConfig>, };

export type SiteConfig = { id: SiteId, label: string, remote_path: string, service_type: ServiceKind, service_name: string | null, web_root: string | null, build_command: string | null, git_remote: string, git_branch: string, local_path: string | null, live_url: string | null, health_check_url: string | null, };

export type ServiceConfig = { "type": "docker", compose_project: string, compose_file: string | null, } | { "type": "systemd", unit: string, } | { "type": "pm2", app_name: string, } | { "type": "static", web_root: string, };

export type SiteStatus = { "state": "running", since: string | null, } | { "state": "stopped" } | { "state": "failed", reason: string, } | { "state": "unknown", reason: string, } | { "state": "checking" };

export type GitStatus = { branch: string, ahead: number, behind: number, changed_files: Array<ChangedFile>, has_conflicts: boolean, remote_url: string, remote_is_ssh: boolean, };

export type ChangedFile = { path: string, change: ChangeKind, staged: boolean, };

export type ChangeKind = "added" | "modified" | "deleted" | "renamed" | "untracked";

export type ServiceKind = "docker" | "systemd" | "pm2" | "static";

export type Capabilities = { can_start_stop: boolean, can_restart: boolean, has_logs: boolean, status_is_reliable: boolean, };

export type HostKeyDecision = { "decision": "trusted" } | { "decision": "unknown", fingerprint: string, } | { "decision": "mismatch", expected: string, got: string, };

export type UserMessage = { headline: string, consequence: string, next_action: NextAction, };

export type NextAction = { "kind": "run_command", command: string, } | { "kind": "open_flow", flow: string, label: string, } | { "kind": "retry" } | { "kind": "advice", text: string, };

export type AppError = { "kind": "ssh", "detail": SshError } | { "kind": "git", "detail": GitError } | { "kind": "adapter", "detail": AdapterError } | { "kind": "config", "detail": ConfigError } | { "kind": "pipeline", "detail": PipelineError };

export type AuthFailureReason = { "reason": "agent_rejected" } | { "reason": "no_agent_socket" } | { "reason": "all_methods_exhausted" };

export type SshError = { "code": "host_unreachable", host: string, detail: string, } | { "code": "auth_failed", reason: AuthFailureReason, } | { "code": "host_key_mismatch", host: string, expected: string, got: string, } | { "code": "host_key_unknown", host: string, fingerprint: string, } | { "code": "key_not_in_agent", path: string, } | { "code": "key_not_found", path: string, } | { "code": "command_failed", command: string, exit_code: number, stderr: string, } | { "code": "connection_lost" } | { "code": "timeout", after_ms: bigint, };

export type GitError = { "code": "merge_conflicts", count: number, files: Array<string>, } | { "code": "detached_head" } | { "code": "no_upstream", branch: string, } | { "code": "https_remote", url: string, } | { "code": "non_ssh_remote", url: string, } | { "code": "push_rejected_non_fast_forward" } | { "code": "push_rejected_permission" } | { "code": "operation", detail: string, };

export type AdapterError = { "code": "unparseable", tool: string, detail: string, } | { "code": "action_failed", action: string, detail: string, } | { "code": "unsupported", operation: string, service_type: string, } | { "code": "ssh" } & SshError;

export type ConfigError = { "code": "unreadable", path: string, detail: string, } | { "code": "malformed", detail: string, } | { "code": "invalid_field", field: string, problem: string, } | { "code": "schema_too_new", found: number, supported: number, };

export type PipelineError = { "code": "step_failed", step: string, detail: string, } | { "code": "cancelled", step: string, mid_mutation: boolean, };

export type Step = "check" | "commit" | "push" | "pull" | "build" | "restart" | "verify";

export type StepState = { "state": "pending" } | { "state": "skipped", reason: string, } | { "state": "running" } | { "state": "ok", summary: string, duration_ms: bigint, } | { "state": "failed", summary: string, duration_ms: bigint, };

export type StepOutcome = "ok" | "failed";

export type PipelineState = { steps: Array<StepEntry>, finished: boolean, rollback_sha: string | null, };

export type StepEntry = { step: Step, state: StepState, };

export type Check = "local_key_exists" | "key_in_agent" | "key_on_github" | "local_remote_is_ssh" | "test_push" | "server_can_pull" | "server_remote_is_ssh";

export type CheckStatus = { "status": "pass" } | { "status": "fail", what_is_wrong: string, } | { "status": "not_applicable", why: string, } | { "status": "running" };

export type FixPreview = { command: string, description: string, undo: string | null, };

export type Fix = { "fix": "generate_local_key", preview: FixPreview, } | { "fix": "convert_remote", preview: FixPreview, };

export type CommandOutcome = { exit_code: number, stdout: string, stderr: string, duration_ms: bigint, command_display: string, };

export type CommandLogEntry = { timestamp: string, server: ServerId, command: string, exit_code: number | null, duration_ms: bigint, };

export type LatestCommit = { short_sha: string, author: string, summary: string, };

export type CiStatus = "passing" | "failing" | "pending" | "none";

