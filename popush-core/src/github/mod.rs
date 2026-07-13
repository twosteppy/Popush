use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LatestCommit {
    pub short_sha: String,
    pub author: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
    Passing,
    Failing,
    Pending,
    #[default]
    None,
}

pub fn parse_latest_commit(json: &str) -> Option<LatestCommit> {
    #[derive(Deserialize)]
    struct Resp {
        sha: String,
        commit: CommitObj,
    }
    #[derive(Deserialize)]
    struct CommitObj {
        message: String,
        author: AuthorObj,
    }
    #[derive(Deserialize)]
    struct AuthorObj {
        name: String,
    }

    let resp: Resp = serde_json::from_str(json).ok()?;
    let short_sha = resp.sha.chars().take(7).collect();
    let summary = resp.commit.message.lines().next().unwrap_or("").to_string();
    Some(LatestCommit {
        short_sha,
        author: resp.commit.author.name,
        summary,
    })
}

pub fn parse_ci_status(json: &str) -> CiStatus {
    #[derive(Deserialize)]
    struct Resp {
        check_runs: Vec<CheckRun>,
    }
    #[derive(Deserialize)]
    struct CheckRun {
        status: String,
        #[serde(default)]
        conclusion: Option<String>,
    }

    let Ok(resp) = serde_json::from_str::<Resp>(json) else {
        return CiStatus::None;
    };
    if resp.check_runs.is_empty() {
        return CiStatus::None;
    }
    if resp.check_runs.iter().any(|c| c.status != "completed") {
        return CiStatus::Pending;
    }
    let all_ok = resp.check_runs.iter().all(|c| {
        matches!(
            c.conclusion.as_deref(),
            Some("success") | Some("neutral") | Some("skipped")
        )
    });
    if all_ok {
        CiStatus::Passing
    } else {
        CiStatus::Failing
    }
}

pub fn parse_open_pr_count(json: &str) -> usize {
    #[derive(Deserialize)]
    struct Pr {}
    serde_json::from_str::<Vec<Pr>>(json)
        .map(|v| v.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT: &str = r#"{
      "sha": "a3f9c21b8e4d5f6a7b8c9d0e1f2a3b4c5d6e7f80",
      "commit": {
        "message": "fix: correct VAT calculation\n\nlonger body here",
        "author": { "name": "David Powers", "email": "d@example.com" }
      }
    }"#;

    #[test]
    fn parses_latest_commit_short_sha_and_summary() {
        let c = parse_latest_commit(COMMIT).unwrap();
        assert_eq!(c.short_sha, "a3f9c21");
        assert_eq!(c.author, "David Powers");
        assert_eq!(c.summary, "fix: correct VAT calculation");
    }

    #[test]
    fn latest_commit_rejects_garbage() {
        assert!(parse_latest_commit("not json").is_none());
    }

    #[test]
    fn ci_all_success_is_passing() {
        let json = r#"{"check_runs":[
          {"status":"completed","conclusion":"success"},
          {"status":"completed","conclusion":"skipped"}
        ]}"#;
        assert_eq!(parse_ci_status(json), CiStatus::Passing);
    }

    #[test]
    fn ci_any_failure_is_failing() {
        let json = r#"{"check_runs":[
          {"status":"completed","conclusion":"success"},
          {"status":"completed","conclusion":"failure"}
        ]}"#;
        assert_eq!(parse_ci_status(json), CiStatus::Failing);
    }

    #[test]
    fn ci_incomplete_is_pending() {
        let json = r#"{"check_runs":[{"status":"in_progress","conclusion":null}]}"#;
        assert_eq!(parse_ci_status(json), CiStatus::Pending);
    }

    #[test]
    fn ci_no_checks_is_none() {
        assert_eq!(parse_ci_status(r#"{"check_runs":[]}"#), CiStatus::None);
        assert_eq!(parse_ci_status("garbage"), CiStatus::None);
    }

    #[test]
    fn counts_open_prs() {
        assert_eq!(parse_open_pr_count("[{},{},{}]"), 3);
        assert_eq!(parse_open_pr_count("[]"), 0);
        assert_eq!(parse_open_pr_count("garbage"), 0);
    }
}
