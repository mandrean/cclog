use std::collections::{BTreeMap, HashMap};

use crate::git::Commit;

/// The second level of the changelog, i.e. the components -> commit information
pub type ComponentMap = BTreeMap<String, Vec<Commit>>;

/// A struct which holds sections to and components->commits map
pub struct SectionMap {
    /// The top level map of the changelog, i.e. sections -> components
    pub sections: HashMap<String, ComponentMap>,
}

impl SectionMap {
    /// Creates a section map from a vector of commits, which we can then
    /// iterate through and write
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::fs::File;
    /// # use cclog::{Clog, SectionMap};
    /// # use cclog::fmt::{FormatWriter, MarkdownWriter};
    /// let clog = Clog::new().unwrap();
    ///
    /// // Get the commits we're interested in...
    /// let sm = SectionMap::from_commits(clog.get_commits().unwrap());
    ///
    /// // Create a file to hold our results, which the MardownWriter will wrap (note, .unwrap() is only
    /// // used to keep the example short and concise)
    /// let mut file = File::create("my_changelog.md").ok().unwrap();
    ///
    /// // Create the MarkdownWriter
    /// let mut writer = MarkdownWriter::new(&mut file);
    ///
    /// // Use the MarkdownWriter to write the changelog
    /// clog.write_changelog_with(&mut writer).unwrap();
    /// ```
    pub fn from_commits(commits: Vec<Commit>) -> SectionMap {
        let mut sm = SectionMap {
            sections: HashMap::new(),
        };

        for entry in commits {
            if !entry.breaks.is_empty() {
                let comp_map = sm
                    .sections
                    .entry("Breaking Changes".to_owned())
                    .or_default();
                let sec_map = comp_map.entry(entry.component.clone()).or_default();
                sec_map.push(entry.clone());
            }
            let comp_map = sm.sections.entry(entry.commit_type.clone()).or_default();
            let sec_map = comp_map.entry(entry.component.clone()).or_default();
            sec_map.push(entry);
        }

        sm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(commit_type: &str, component: &str, subject: &str) -> Commit {
        Commit {
            hash: "abc12345".to_owned(),
            subject: subject.to_owned(),
            component: component.to_owned(),
            closes: vec![],
            breaks: vec![],
            commit_type: commit_type.to_owned(),
        }
    }

    #[test]
    fn groups_by_type_and_component() {
        let commits = vec![
            commit("Features", "parser", "add parsing"),
            commit("Features", "cli", "add flag"),
            commit("Bug Fixes", "parser", "fix crash"),
        ];
        let sm = SectionMap::from_commits(commits);

        assert!(sm.sections.contains_key("Features"));
        assert!(sm.sections.contains_key("Bug Fixes"));

        let features = &sm.sections["Features"];
        assert!(features.contains_key("parser"));
        assert!(features.contains_key("cli"));
        assert_eq!(features["parser"].len(), 1);
        assert_eq!(features["cli"].len(), 1);

        let fixes = &sm.sections["Bug Fixes"];
        assert_eq!(fixes["parser"].len(), 1);
    }

    #[test]
    fn breaking_changes_creates_separate_section() {
        let commits = vec![Commit {
            hash: "abc12345".to_owned(),
            subject: "change api".to_owned(),
            component: "api".to_owned(),
            closes: vec![],
            breaks: vec!["10".to_owned()],
            commit_type: "Features".to_owned(),
        }];
        let sm = SectionMap::from_commits(commits);

        // Should appear in both "Features" and "Breaking Changes"
        assert!(sm.sections.contains_key("Features"));
        assert!(sm.sections.contains_key("Breaking Changes"));
        assert_eq!(sm.sections["Breaking Changes"]["api"].len(), 1);
    }

    #[test]
    fn empty_commits() {
        let sm = SectionMap::from_commits(vec![]);
        assert!(sm.sections.is_empty());
    }

    #[test]
    fn multiple_commits_same_component() {
        let commits = vec![
            commit("Features", "ui", "add button"),
            commit("Features", "ui", "add dialog"),
        ];
        let sm = SectionMap::from_commits(commits);
        assert_eq!(sm.sections["Features"]["ui"].len(), 2);
    }
}
