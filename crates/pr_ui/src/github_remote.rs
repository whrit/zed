/// Represents a GitHub repository with owner and repo name
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRepo {
    pub owner: String,
    pub repo: String,
}

impl GitHubRepo {
    /// Parses GitHub owner and repo from various remote URL formats
    /// Supports:
    /// - https://github.com/owner/repo.git
    /// - https://github.com/owner/repo
    /// - git@github.com:owner/repo.git
    /// - git@github.com:owner/repo
    /// - ssh://git@github.com/owner/repo.git
    pub fn from_remote_url(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        if url.contains("github.com") {
            if url.starts_with("http://") || url.starts_with("https://") {
                Self::parse_https_url(url)
            } else if url.starts_with("ssh://") {
                Self::parse_ssh_protocol_url(url)
            } else if url.contains('@') && url.contains(':') {
                Self::parse_scp_style_url(url)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the full repository name in "owner/repo" format
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    fn parse_https_url(url: &str) -> Option<Self> {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 5 {
            return None;
        }

        let github_com_index = parts.iter().position(|&p| p == "github.com")?;
        if github_com_index + 2 >= parts.len() {
            return None;
        }

        let owner = parts[github_com_index + 1];
        let mut repo = parts[github_com_index + 2];

        repo = repo.trim_end_matches('/');
        repo = repo.trim_end_matches(".git");

        if owner.is_empty() || repo.is_empty() {
            return None;
        }

        if parts.len() > github_com_index + 3 {
            return None;
        }

        Some(GitHubRepo {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    fn parse_ssh_protocol_url(url: &str) -> Option<Self> {
        let without_protocol = url.strip_prefix("ssh://")?;
        let path_start = without_protocol.find('/')?;
        let path = &without_protocol[path_start + 1..];

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return None;
        }

        let owner = parts[0];
        let mut repo = parts[1];

        repo = repo.trim_end_matches('/');
        repo = repo.trim_end_matches(".git");

        if owner.is_empty() || repo.is_empty() {
            return None;
        }

        if parts.len() > 2 {
            return None;
        }

        Some(GitHubRepo {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    fn parse_scp_style_url(url: &str) -> Option<Self> {
        let at_index = url.find('@')?;
        let colon_index = url.find(':')?;

        if colon_index <= at_index {
            return None;
        }

        let host = &url[at_index + 1..colon_index];
        if host != "github.com" {
            return None;
        }

        let path = &url[colon_index + 1..];
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() < 2 {
            return None;
        }

        let owner = parts[0];
        let mut repo = parts[1];

        repo = repo.trim_end_matches('/');
        repo = repo.trim_end_matches(".git");

        if owner.is_empty() || repo.is_empty() {
            return None;
        }

        if parts.len() > 2 {
            return None;
        }

        Some(GitHubRepo {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_https_url_with_git_suffix() {
        let repo = GitHubRepo::from_remote_url("https://github.com/zed-industries/zed.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "zed-industries".into(),
                repo: "zed".into()
            })
        );
    }

    #[test]
    fn test_https_url_without_git_suffix() {
        let repo = GitHubRepo::from_remote_url("https://github.com/octocat/hello-world");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "octocat".into(),
                repo: "hello-world".into()
            })
        );
    }

    #[test]
    fn test_ssh_url_with_git_suffix() {
        let repo = GitHubRepo::from_remote_url("git@github.com:zed-industries/zed.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "zed-industries".into(),
                repo: "zed".into()
            })
        );
    }

    #[test]
    fn test_ssh_url_without_git_suffix() {
        let repo = GitHubRepo::from_remote_url("git@github.com:octocat/hello-world");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "octocat".into(),
                repo: "hello-world".into()
            })
        );
    }

    #[test]
    fn test_ssh_protocol_url() {
        let repo = GitHubRepo::from_remote_url("ssh://git@github.com/zed-industries/zed.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "zed-industries".into(),
                repo: "zed".into()
            })
        );
    }

    #[test]
    fn test_ssh_protocol_url_without_git_suffix() {
        let repo = GitHubRepo::from_remote_url("ssh://git@github.com/octocat/hello-world");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "octocat".into(),
                repo: "hello-world".into()
            })
        );
    }

    #[test]
    fn test_owner_with_dashes_and_underscores() {
        let repo = GitHubRepo::from_remote_url("https://github.com/my-awesome_org/cool-repo.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "my-awesome_org".into(),
                repo: "cool-repo".into()
            })
        );
    }

    #[test]
    fn test_repo_with_dots() {
        let repo = GitHubRepo::from_remote_url("https://github.com/user/repo.name.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "user".into(),
                repo: "repo.name".into()
            })
        );
    }

    #[test]
    fn test_non_github_url_returns_none() {
        let repo = GitHubRepo::from_remote_url("https://gitlab.com/user/repo.git");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_invalid_url_returns_none() {
        let repo = GitHubRepo::from_remote_url("not_a_url");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_github_url_with_trailing_slash() {
        let repo = GitHubRepo::from_remote_url("https://github.com/user/repo/");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "user".into(),
                repo: "repo".into()
            })
        );
    }

    #[test]
    fn test_github_url_with_path_segments() {
        let repo = GitHubRepo::from_remote_url("https://github.com/user/repo/tree/main");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_ssh_with_custom_username() {
        let repo = GitHubRepo::from_remote_url("custom-user@github.com:owner/repo.git");
        assert_eq!(
            repo,
            Some(GitHubRepo {
                owner: "owner".into(),
                repo: "repo".into()
            })
        );
    }

    #[test]
    fn test_full_name_method() {
        let repo = GitHubRepo {
            owner: "test-owner".into(),
            repo: "test-repo".into(),
        };
        assert_eq!(repo.full_name(), "test-owner/test-repo");
    }

    #[test]
    fn test_empty_url() {
        let repo = GitHubRepo::from_remote_url("");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_github_url_missing_repo() {
        let repo = GitHubRepo::from_remote_url("https://github.com/owner");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_github_url_missing_owner() {
        let repo = GitHubRepo::from_remote_url("https://github.com/");
        assert_eq!(repo, None);
    }

    #[test]
    fn test_ssh_url_missing_repo() {
        let repo = GitHubRepo::from_remote_url("git@github.com:owner");
        assert_eq!(repo, None);
    }
}
