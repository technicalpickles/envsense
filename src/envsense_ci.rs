use ci_info::{get, types::Vendor};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct CiFacet {
    pub is_ci: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_lower = true;
        } else {
            if !out.ends_with('_') {
                out.push('_');
            }
            prev_lower = false;
        }
    }
    out
}

pub fn normalize_vendor(v: Vendor) -> (String, String) {
    use Vendor::*;
    match v {
        GitHubActions => ("github_actions".into(), "GitHub Actions".into()),
        GitLabCI => ("gitlab_ci".into(), "GitLab CI".into()),
        CircleCI => ("circleci".into(), "CircleCI".into()),
        Buildkite => ("buildkite".into(), "Buildkite".into()),
        Jenkins => ("jenkins".into(), "Jenkins".into()),
        TeamCity => ("teamcity".into(), "TeamCity".into()),
        BitbucketPipelines => ("bitbucket_pipelines".into(), "Bitbucket Pipelines".into()),
        AzurePipelines => ("azure_pipelines".into(), "Azure Pipelines".into()),
        GoogleCloudBuild => ("gcb".into(), "Google Cloud Build".into()),
        Vercel => ("vercel".into(), "Vercel".into()),
        AWSCodeBuild => ("aws_codebuild".into(), "AWS CodeBuild".into()),
        SourceHut => ("sourcehut".into(), "SourceHut".into()),
        AppVeyor => ("appveyor".into(), "AppVeyor".into()),
        other => {
            let raw = format!("{:?}", other);
            let id = to_snake_case(&raw);
            (id, raw)
        }
    }
}

pub fn detect_ci() -> CiFacet {
    let info = get();
    if !info.ci {
        return CiFacet::default();
    }
    let (vendor, name) = info
        .vendor
        .map(normalize_vendor)
        .unwrap_or_else(|| ("generic".into(), "Generic CI".into()));
    CiFacet {
        is_ci: true,
        vendor: Some(vendor),
        name: Some(name),
        pr: info.pr,
        branch: info.branch_name,
    }
}

pub fn ci_traits(f: &CiFacet) -> Vec<(String, Value)> {
    let mut out = vec![("is_ci".into(), json!(f.is_ci))];
    if let Some(v) = &f.vendor {
        out.push(("ci_vendor".into(), json!(v)));
    }
    if let Some(n) = &f.name {
        out.push(("ci_name".into(), json!(n)));
    }
    if let Some(p) = f.pr {
        out.push(("ci_pr".into(), json!(p)));
    }
    if let Some(b) = &f.branch {
        out.push(("ci_branch".into(), json!(b)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn normalize_known_vendors() {
        let cases = [
            (Vendor::GitHubActions, "github_actions", "GitHub Actions"),
            (Vendor::GitLabCI, "gitlab_ci", "GitLab CI"),
            (Vendor::CircleCI, "circleci", "CircleCI"),
            (Vendor::Buildkite, "buildkite", "Buildkite"),
            (Vendor::Jenkins, "jenkins", "Jenkins"),
            (Vendor::TeamCity, "teamcity", "TeamCity"),
            (
                Vendor::BitbucketPipelines,
                "bitbucket_pipelines",
                "Bitbucket Pipelines",
            ),
            (Vendor::AzurePipelines, "azure_pipelines", "Azure Pipelines"),
            (Vendor::GoogleCloudBuild, "gcb", "Google Cloud Build"),
            (Vendor::Vercel, "vercel", "Vercel"),
            (Vendor::AWSCodeBuild, "aws_codebuild", "AWS CodeBuild"),
            (Vendor::SourceHut, "sourcehut", "SourceHut"),
            (Vendor::AppVeyor, "appveyor", "AppVeyor"),
        ];
        for (v, id, name) in cases {
            assert_eq!(normalize_vendor(v), (id.into(), name.into()));
        }
    }

    #[test]
    #[serial]
    fn generic_fallback_when_ci_true_but_no_vendor() {
        unsafe {
            std::env::set_var("CI", "1");
            std::env::remove_var("GITHUB_ACTIONS");
        }
        let ci = detect_ci();
        assert!(ci.is_ci);
        assert_eq!(ci.vendor.as_deref(), Some("generic"));
        assert_eq!(ci.name.as_deref(), Some("Generic CI"));
        unsafe {
            std::env::remove_var("CI");
        }
    }

    #[test]
    #[serial]
    fn non_ci_case() {
        unsafe {
            std::env::remove_var("CI");
            std::env::remove_var("GITHUB_ACTIONS");
        }
        let ci = detect_ci();
        assert!(!ci.is_ci);
        assert!(ci.vendor.is_none());
        assert!(ci.name.is_none());
        assert!(ci.pr.is_none());
        assert!(ci.branch.is_none());
    }
}
