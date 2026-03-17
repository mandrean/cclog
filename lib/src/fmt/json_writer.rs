use std::io;

use serde::Serialize;
use time::OffsetDateTime;

use crate::{clog::Clog, error::Result, fmt::FormatWriter, git::Commit, sectionmap::SectionMap};

#[derive(Serialize)]
struct Changelog {
    header: Header,
    sections: Option<Vec<Section>>,
}

#[derive(Serialize)]
struct Header {
    version: Option<String>,
    patch_version: bool,
    subtitle: Option<String>,
    date: String,
}

#[derive(Serialize)]
struct Section {
    title: String,
    commits: Option<Vec<CommitEntry>>,
}

#[derive(Serialize)]
struct CommitEntry {
    component: Option<String>,
    subject: String,
    commit_link: String,
    closes: Option<Vec<IssueRef>>,
    breaks: Option<Vec<IssueRef>>,
}

#[derive(Serialize)]
struct IssueRef {
    issue: String,
    issue_link: String,
}

/// Wraps a `std::io::Write` object to write `clog` output in a JSON format
///
/// # Example
///
/// ```no_run
/// # use std::fs::File;
/// # use cclog::{SectionMap, Clog, fmt::JsonWriter};
/// let clog = Clog::new().unwrap();
///
/// // Get the commits we're interested in...
/// let sm = SectionMap::from_commits(clog.get_commits().unwrap());
///
/// // Create a file to hold our results, which the JsonWriter will wrap (note, .unwrap() is only
/// // used to keep the example short and concise)
/// let mut file = File::create("my_changelog.json").ok().unwrap();
///
/// // Create the JSON Writer
/// let mut writer = JsonWriter::new(&mut file);
///
/// // Use the JsonWriter to write the changelog
/// clog.write_changelog_with(&mut writer).unwrap();
/// ```
pub struct JsonWriter<'a>(&'a mut dyn io::Write);

impl<'a> JsonWriter<'a> {
    /// Creates a new instance of the `JsonWriter` struct using a
    /// `std::io::Write` object.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::io::{stdout, BufWriter};
    /// # use cclog::{Clog, fmt::JsonWriter};
    /// let clog = Clog::new().unwrap();
    ///
    /// // Create a JsonWriter to wrap stdout
    /// let out = stdout();
    /// let mut out_buf = BufWriter::new(out.lock());
    /// let mut writer = JsonWriter::new(&mut out_buf);
    /// ```
    pub fn new<T: io::Write>(writer: &'a mut T) -> JsonWriter<'a> {
        JsonWriter(writer)
    }

    fn build_issue_refs(issues: &[String], options: &Clog) -> Option<Vec<IssueRef>> {
        if issues.is_empty() {
            return None;
        }
        Some(
            issues
                .iter()
                .map(|issue| IssueRef {
                    issue: issue.clone(),
                    issue_link: options.link_style.issue_link(issue, options.repo.as_ref()),
                })
                .collect(),
        )
    }

    fn build_commit_entry(entry: &Commit, options: &Clog) -> CommitEntry {
        CommitEntry {
            component: if entry.component.is_empty() {
                None
            } else {
                Some(entry.component.clone())
            },
            subject: entry.subject.clone(),
            commit_link: options
                .link_style
                .commit_link(&entry.hash, options.repo.as_ref()),
            closes: Self::build_issue_refs(&entry.closes, options),
            breaks: Self::build_issue_refs(&entry.breaks, options),
        }
    }

    fn build_section(
        title: &str,
        compmap: &crate::sectionmap::ComponentMap,
        options: &Clog,
    ) -> Section {
        let commits: Vec<CommitEntry> = compmap
            .values()
            .flat_map(|entries| entries.iter().map(|e| Self::build_commit_entry(e, options)))
            .collect();

        Section {
            title: title.to_owned(),
            commits: if commits.is_empty() {
                None
            } else {
                Some(commits)
            },
        }
    }
}

impl FormatWriter for JsonWriter<'_> {
    fn write_changelog(&mut self, options: &Clog, sm: &SectionMap) -> Result<()> {
        let now = OffsetDateTime::now_utc();
        let date = now.format(&time::format_description::parse("[year]-[month]-[day]").unwrap())?;

        let sections: Vec<Section> = options
            .section_map
            .keys()
            .filter_map(|sec| {
                sm.sections
                    .get(sec)
                    .map(|compmap| Self::build_section(sec, compmap, options))
            })
            .collect();

        let changelog = Changelog {
            header: Header {
                version: options.version.clone(),
                patch_version: options.patch_ver,
                subtitle: options.subtitle.clone(),
                date,
            },
            sections: if sections.is_empty() {
                None
            } else {
                Some(sections)
            },
        };

        serde_json::to_writer(&mut self.0, &changelog)
            .map_err(|e| crate::error::Error::Io(e.into()))?;
        self.0.flush().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_clog() -> Clog {
        Clog::default()
            .repository("https://github.com/test/repo")
            .version("1.0.0")
            .subtitle("Release")
    }

    #[test]
    fn write_changelog_with_subtitle() {
        let clog = test_clog();
        let sm = SectionMap::from_commits(vec![]);
        let mut buf = Vec::new();
        let mut writer = JsonWriter::new(&mut buf);
        writer.write_changelog(&clog, &sm).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(v["header"]["subtitle"], "Release");
        assert_eq!(v["header"]["version"], "1.0.0");
        assert_eq!(v["header"]["patch_version"], false);
        assert!(v["sections"].is_null());
    }

    #[test]
    fn write_changelog_null_subtitle() {
        let clog = Clog::default().version("1.0.0");
        let sm = SectionMap::from_commits(vec![]);
        let mut buf = Vec::new();
        let mut writer = JsonWriter::new(&mut buf);
        writer.write_changelog(&clog, &sm).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert!(v["header"]["subtitle"].is_null());
    }

    #[test]
    fn write_changelog_with_commits() {
        let clog = test_clog();
        let commits = vec![
            Commit {
                hash: "abc1234567890abcdef1234567890abcdef123456".to_owned(),
                subject: "change api".to_owned(),
                component: "api".to_owned(),
                closes: vec!["99".to_owned()],
                breaks: vec!["42".to_owned()],
                commit_type: "Features".to_owned(),
            },
            Commit {
                hash: "def1234567890abcdef1234567890abcdef123456".to_owned(),
                subject: "fix crash".to_owned(),
                component: "".to_owned(),
                closes: vec![],
                breaks: vec![],
                commit_type: "Bug Fixes".to_owned(),
            },
        ];
        let sm = SectionMap::from_commits(commits);

        let mut buf = Vec::new();
        let mut writer = JsonWriter::new(&mut buf);
        writer.write_changelog(&clog, &sm).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let v: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Verify it's valid JSON
        assert!(v["sections"].is_array());

        // Find the Features section
        let sections = v["sections"].as_array().unwrap();
        let features = sections.iter().find(|s| s["title"] == "Features").unwrap();
        let commits = features["commits"].as_array().unwrap();
        assert_eq!(commits[0]["component"], "api");
        assert_eq!(commits[0]["subject"], "change api");

        // Verify breaks contains issue 42, not 99
        assert_eq!(commits[0]["breaks"][0]["issue"], "42");
        // Verify closes contains issue 99
        assert_eq!(commits[0]["closes"][0]["issue"], "99");

        // Find the Bug Fixes section
        let fixes = sections.iter().find(|s| s["title"] == "Bug Fixes").unwrap();
        let fix_commits = fixes["commits"].as_array().unwrap();
        assert!(fix_commits[0]["component"].is_null());
        assert!(fix_commits[0]["closes"].is_null());
        assert!(fix_commits[0]["breaks"].is_null());
    }

    #[test]
    fn output_is_valid_json() {
        let clog = test_clog();
        let sm = SectionMap::from_commits(vec![]);
        let mut buf = Vec::new();
        let mut writer = JsonWriter::new(&mut buf);
        writer.write_changelog(&clog, &sm).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Must parse as valid JSON
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(result.is_ok(), "Invalid JSON: {output}");
    }
}
