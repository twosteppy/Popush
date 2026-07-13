use crate::error::{NextAction, UserMessage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    PushNonFastForward,
    PushPermissionDenied,
    PullLocalChangesOnServer { remote_path: String },
    BuildFailed { exit_code: i32, output: String },
    RestartFailed { service_logs: String },
    VerifyHealthCheck { code: u16, logs: String },
}

pub fn failure_message(kind: &FailureKind) -> UserMessage {
    match kind {
        FailureKind::PushNonFastForward => UserMessage {
            headline: "Push rejected. The remote has commits you do not have.".into(),
            consequence: "Your push did not go through; the remote is ahead of you.".into(),
            next_action: NextAction::Advice {
                text: "Pull and merge them first.".into(),
            },
        },
        FailureKind::PushPermissionDenied => UserMessage {
            headline: "Push rejected: permission denied.".into(),
            consequence: "Your SSH key may not be registered with GitHub.".into(),
            next_action: NextAction::OpenFlow {
                flow: "wizard".into(),
                label: "Run the setup wizard".into(),
            },
        },
        FailureKind::PullLocalChangesOnServer { remote_path } => UserMessage {
            headline: format!(
                "Pull failed. There are uncommitted changes on the server in `{remote_path}`."
            ),
            consequence: "Someone edited files directly on the VPS. Popush will not overwrite them."
                .into(),
            next_action: NextAction::Advice {
                text: "Commit or discard those changes on the server, then try again.".into(),
            },
        },
        FailureKind::BuildFailed { exit_code, output } => UserMessage {
            headline: format!("Build failed with exit code {exit_code}."),
            consequence:
                "The code is on the server but has not been deployed. Your site is still running the previous version."
                    .into(),
            next_action: NextAction::Advice {
                text: output.clone(),
            },
        },
        FailureKind::RestartFailed { service_logs } => UserMessage {
            headline: "Restart failed. The new code is built but the service did not come back up."
                .into(),
            consequence: "Your site may be down.".into(),
            next_action: NextAction::Advice {
                text: service_logs.clone(),
            },
        },
        FailureKind::VerifyHealthCheck { code, logs } => UserMessage {
            headline: format!(
                "The service restarted, but the health check returned {code}."
            ),
            consequence: "The deploy technically succeeded but the site is erroring.".into(),
            next_action: NextAction::Advice { text: logs.clone() },
        },
    }
}

pub fn rollback_offer(remote_path: &str, sha: &str) -> UserMessage {
    UserMessage {
        headline: format!("Previous version was `{sha}`."),
        consequence: "Popush made no automatic rollback; that is deliberate, so it never undoes work you did not expect.".into(),
        next_action: NextAction::RunCommand {
            command: format!("cd {remote_path} && git reset --hard {sha}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANNED: &[&str] = &["Deploy failed", "Something went wrong"];

    fn all_message_texts(m: &UserMessage) -> Vec<String> {
        let mut v = vec![m.headline.clone(), m.consequence.clone()];
        match &m.next_action {
            NextAction::RunCommand { command } => v.push(command.clone()),
            NextAction::OpenFlow { flow, label } => {
                v.push(flow.clone());
                v.push(label.clone());
            }
            NextAction::Retry => {}
            NextAction::Advice { text } => v.push(text.clone()),
        }
        v
    }

    #[test]
    fn push_non_fast_forward_is_verbatim() {
        let m = failure_message(&FailureKind::PushNonFastForward);
        assert_eq!(
            m.headline,
            "Push rejected. The remote has commits you do not have."
        );
        assert!(all_message_texts(&m)
            .iter()
            .any(|t| t == "Pull and merge them first."));
    }

    #[test]
    fn push_permission_is_verbatim_with_wizard_button() {
        let m = failure_message(&FailureKind::PushPermissionDenied);
        assert_eq!(m.headline, "Push rejected: permission denied.");
        assert!(m
            .consequence
            .contains("SSH key may not be registered with GitHub"));
        assert_eq!(
            m.next_action,
            NextAction::OpenFlow {
                flow: "wizard".into(),
                label: "Run the setup wizard".into()
            }
        );
    }

    #[test]
    fn pull_local_changes_names_the_real_path() {
        let m = failure_message(&FailureKind::PullLocalChangesOnServer {
            remote_path: "/srv/site".into(),
        });
        assert!(m.headline.contains("`/srv/site`"));
        assert!(m
            .consequence
            .contains("Someone edited files directly on the VPS"));
    }

    #[test]
    fn build_failed_reports_real_exit_code_and_keeps_previous_version_language() {
        let m = failure_message(&FailureKind::BuildFailed {
            exit_code: 1,
            output: "error: type mismatch".into(),
        });
        assert_eq!(m.headline, "Build failed with exit code 1.");
        assert!(m.consequence.contains("still running the previous version"));
    }

    #[test]
    fn restart_failed_is_verbatim() {
        let m = failure_message(&FailureKind::RestartFailed {
            service_logs: "oom killed".into(),
        });
        assert_eq!(
            m.headline,
            "Restart failed. The new code is built but the service did not come back up."
        );
        assert_eq!(m.consequence, "Your site may be down.");
    }

    #[test]
    fn verify_health_check_reports_real_code() {
        let m = failure_message(&FailureKind::VerifyHealthCheck {
            code: 500,
            logs: "panic".into(),
        });
        assert_eq!(
            m.headline,
            "The service restarted, but the health check returned 500."
        );
    }

    #[test]
    fn no_failure_message_contains_a_banned_string() {
        let kinds = [
            FailureKind::PushNonFastForward,
            FailureKind::PushPermissionDenied,
            FailureKind::PullLocalChangesOnServer {
                remote_path: "/srv".into(),
            },
            FailureKind::BuildFailed {
                exit_code: 1,
                output: "x".into(),
            },
            FailureKind::RestartFailed {
                service_logs: "x".into(),
            },
            FailureKind::VerifyHealthCheck {
                code: 500,
                logs: "x".into(),
            },
        ];
        for kind in &kinds {
            let m = failure_message(kind);
            for text in all_message_texts(&m) {
                for banned in BANNED {
                    assert!(
                        !text.contains(banned),
                        "banned string {banned:?} in message {text:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn rollback_offer_shows_sha_and_command() {
        let m = rollback_offer("/srv/site", "a3f9c21");
        assert!(m.headline.contains("a3f9c21"));
        assert_eq!(
            m.next_action,
            NextAction::RunCommand {
                command: "cd /srv/site && git reset --hard a3f9c21".into()
            }
        );
    }
}
