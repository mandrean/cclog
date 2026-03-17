use std::str::FromStr;

use strum::{Display, EnumString};

/// Determines the hyperlink style used in commit and issue links. Defaults to
/// `LinksStyle::Github`
///
/// # Example
///
/// ```no_run
/// # use cclog::{LinkStyle, Clog};
/// let clog = Clog::new().unwrap();
/// clog.link_style(LinkStyle::Stash);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum LinkStyle {
    #[default]
    Github,
    Gitlab,
    Stash,
    Cgit,
}

impl<'de> serde::de::Deserialize<'de> for LinkStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl LinkStyle {
    /// Gets a hyperlink url to an issue in the specified format.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use cclog::{LinkStyle, Clog};
    /// let link = LinkStyle::Github;
    /// let issue = link.issue_link("141", Some("https://github.com/thoughtram/clog"));
    ///
    /// assert_eq!("https://github.com/thoughtram/clog/issues/141", issue);
    /// ```
    pub fn issue_link<S: AsRef<str>>(&self, issue: S, repo: Option<S>) -> String {
        let issue = issue.as_ref();
        if let Some(link) = repo {
            let link = link.as_ref();
            match *self {
                LinkStyle::Github | LinkStyle::Gitlab => format!("{link}/issues/{issue}"),
                // cgit does not support issues
                LinkStyle::Stash | LinkStyle::Cgit => issue.to_string(),
            }
        } else {
            issue.to_string()
        }
    }

    /// Gets a hyperlink url to a commit in the specified format.
    ///
    /// # Example
    /// ```no_run
    /// # use cclog::{LinkStyle, Clog};
    /// let link = LinkStyle::Github;
    /// let commit = link.commit_link(
    ///     "123abc891234567890abcdefabc4567898724",
    ///     Some("https://github.com/clog-tool/clog-lib"),
    /// );
    ///
    /// assert_eq!(
    ///     "https://github.com/thoughtram/clog/commit/123abc891234567890abcdefabc4567898724",
    ///     commit
    /// );
    /// ```
    pub fn commit_link<S: AsRef<str>>(&self, hash: S, repo: Option<S>) -> String {
        let hash = hash.as_ref();
        if let Some(link) = repo {
            let link = link.as_ref();
            match *self {
                LinkStyle::Github | LinkStyle::Gitlab => format!("{link}/commit/{hash}"),
                LinkStyle::Stash => format!("{link}/commits/{hash}"),
                LinkStyle::Cgit => format!("{link}/commit/?id={hash}"),
            }
        } else {
            hash.get(0..8).unwrap_or(hash).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "https://github.com/test/repo";
    const HASH: &str = "abc1234567890abcdef";

    #[test]
    fn github_issue_link() {
        let link = LinkStyle::Github.issue_link("42", Some(REPO));
        assert_eq!(link, "https://github.com/test/repo/issues/42");
    }

    #[test]
    fn gitlab_issue_link() {
        let link = LinkStyle::Gitlab.issue_link("42", Some(REPO));
        assert_eq!(link, "https://github.com/test/repo/issues/42");
    }

    #[test]
    fn stash_issue_link_returns_number() {
        let link = LinkStyle::Stash.issue_link("42", Some(REPO));
        assert_eq!(link, "42");
    }

    #[test]
    fn cgit_issue_link_returns_number() {
        let link = LinkStyle::Cgit.issue_link("42", Some(REPO));
        assert_eq!(link, "42");
    }

    #[test]
    fn issue_link_without_repo() {
        let link = LinkStyle::Github.issue_link("42", None::<&str>);
        assert_eq!(link, "42");
    }

    #[test]
    fn github_commit_link() {
        let link = LinkStyle::Github.commit_link(HASH, Some(REPO));
        assert_eq!(link, format!("{REPO}/commit/{HASH}"));
    }

    #[test]
    fn stash_commit_link() {
        let link = LinkStyle::Stash.commit_link(HASH, Some(REPO));
        assert_eq!(link, format!("{REPO}/commits/{HASH}"));
    }

    #[test]
    fn cgit_commit_link() {
        let link = LinkStyle::Cgit.commit_link(HASH, Some(REPO));
        assert_eq!(link, format!("{REPO}/commit/?id={HASH}"));
    }

    #[test]
    fn commit_link_without_repo_truncates_hash() {
        let link = LinkStyle::Github.commit_link(HASH, None::<&str>);
        assert_eq!(link, "abc12345");
    }

    #[test]
    fn commit_link_without_repo_short_hash() {
        let link = LinkStyle::Github.commit_link("abc", None::<&str>);
        assert_eq!(link, "abc");
    }

    #[test]
    fn link_style_from_str() {
        assert_eq!("Github".parse::<LinkStyle>().unwrap(), LinkStyle::Github);
        assert_eq!("github".parse::<LinkStyle>().unwrap(), LinkStyle::Github);
        assert_eq!("GITLAB".parse::<LinkStyle>().unwrap(), LinkStyle::Gitlab);
        assert_eq!("stash".parse::<LinkStyle>().unwrap(), LinkStyle::Stash);
        assert_eq!("Cgit".parse::<LinkStyle>().unwrap(), LinkStyle::Cgit);
    }
}
