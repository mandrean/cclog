use std::{collections::BTreeMap, io};

use time::OffsetDateTime;

use crate::{clog::Clog, error::Result, fmt::FormatWriter, git::Commit, sectionmap::SectionMap};

/// Wraps a `std::io::Write` object to write `clog` output in a Markdown format
///
/// # Example
///
/// ```no_run
/// # use std::fs::File;
/// # use cclog::{SectionMap, Clog, fmt::MarkdownWriter};
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
pub struct MarkdownWriter<'a>(&'a mut dyn io::Write);

impl<'a> MarkdownWriter<'a> {
    /// Creates a new instance of the `MarkdownWriter` struct using a
    /// `std::io::Write` object.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::io::BufWriter;
    /// # use cclog::{Clog, fmt::MarkdownWriter};
    /// let clog = Clog::new().unwrap();
    ///
    /// // Create a MarkdownWriter to wrap stdout
    /// let out = std::io::stdout();
    /// let mut out_buf = BufWriter::new(out.lock());
    /// let mut writer = MarkdownWriter::new(&mut out_buf);
    /// ```
    pub fn new<T: io::Write + 'a>(writer: &'a mut T) -> MarkdownWriter<'a> {
        MarkdownWriter(writer)
    }

    fn write_header(&mut self, options: &Clog) -> Result<()> {
        let subtitle = options.subtitle.clone().unwrap_or_default();
        let version = options.version.clone().unwrap_or_default();

        let version_text = if options.patch_ver {
            format!("### {version} {subtitle}")
        } else {
            format!("## {version} {subtitle}")
        };

        let now = OffsetDateTime::now_utc();
        // unwrap because the format description is static
        let date = now.format(&time::format_description::parse("[year]-[month]-[day]").unwrap())?;
        writeln!(
            self.0,
            "<a name=\"{version}\"></a>\n{version_text} ({date})\n",
        )
        .map_err(Into::into)
    }

    /// Writes a particular section of a changelog
    fn write_section(
        &mut self,
        options: &Clog,
        title: &str,
        section: &BTreeMap<&String, &Vec<Commit>>,
    ) -> Result<()> {
        if section.is_empty() {
            return Ok(());
        }

        self.0
            .write_all(format!("\n#### {title}\n\n")[..].as_bytes())?;

        for (component, entries) in section.iter() {
            let nested = (entries.len() > 1) && !component.is_empty();

            let prefix = if nested {
                writeln!(self.0, "* **{component}:**")?;
                "  *".to_owned()
            } else if !component.is_empty() {
                format!("* **{component}:**")
            } else {
                "* ".to_string()
            };

            for entry in entries.iter() {
                write!(
                    self.0,
                    "{prefix} {} ([{}]({})",
                    entry.subject,
                    &entry.hash[0..8],
                    options
                        .link_style
                        .commit_link(&*entry.hash, options.repo.as_deref())
                )?;

                if !entry.closes.is_empty() {
                    let closes_string = entry
                        .closes
                        .iter()
                        .map(|s| {
                            format!(
                                "[#{s}]({})",
                                options.link_style.issue_link(s, options.repo.as_ref())
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    write!(self.0, ", closes {closes_string}")?;
                }
                if !entry.breaks.is_empty() {
                    let breaks_string = entry
                        .breaks
                        .iter()
                        .map(|s| {
                            format!(
                                "[#{s}]({})",
                                options.link_style.issue_link(s, options.repo.as_ref())
                            )
                        })
                        .collect::<Vec<String>>()
                        .join(", ");

                    // 5 = "[#]()" i.e. a commit message that only said "BREAKING"
                    if breaks_string.len() != 5 {
                        write!(self.0, ", breaks {breaks_string}")?;
                    }
                }

                writeln!(self.0, ")")?;
            }
        }

        Ok(())
    }

    /// Writes some contents to the `Write` writer object
    #[allow(dead_code)]
    fn write(&mut self, content: &str) -> Result<()> {
        write!(self.0, "\n\n\n")?;
        write!(self.0, "{}", content).map_err(Into::into)
    }
}

impl FormatWriter for MarkdownWriter<'_> {
    fn write_changelog(&mut self, options: &Clog, sm: &SectionMap) -> Result<()> {
        self.write_header(options)?;

        // Get the section names ordered from `options.section_map`
        let s_it = options
            .section_map
            .keys()
            .filter_map(|sec| sm.sections.get(sec).map(|secmap| (sec, secmap)));
        for (sec, secmap) in s_it {
            self.write_section(
                options,
                &sec[..],
                &secmap.iter().collect::<BTreeMap<_, _>>(),
            )?;
        }

        self.0.flush().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::Commit;

    fn test_clog() -> Clog {
        Clog::default()
            .repository("https://github.com/test/repo")
            .version("1.0.0")
    }

    fn test_commit(component: &str, subject: &str) -> Commit {
        Commit {
            hash: "abc1234567890abcdef1234567890abcdef123456".to_owned(),
            subject: subject.to_owned(),
            component: component.to_owned(),
            closes: vec![],
            breaks: vec![],
            commit_type: "Features".to_owned(),
        }
    }

    #[test]
    fn write_header_normal_release() {
        let clog = test_clog();
        let mut buf = Vec::new();
        let mut writer = MarkdownWriter::new(&mut buf);
        writer.write_header(&clog).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<a name=\"1.0.0\"></a>"));
        assert!(output.contains("## 1.0.0"));
        assert!(!output.contains("### 1.0.0"));
    }

    #[test]
    fn write_header_patch_release() {
        let clog = test_clog().patch_ver(true);
        let mut buf = Vec::new();
        let mut writer = MarkdownWriter::new(&mut buf);
        writer.write_header(&clog).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("### 1.0.0"));
    }

    #[test]
    fn write_section_with_commits() {
        let clog = test_clog();
        let commits = vec![test_commit("parser", "add feature")];
        let component_key = "parser".to_owned();
        let mut section = BTreeMap::new();
        section.insert(&component_key, &commits);

        let mut buf = Vec::new();
        let mut writer = MarkdownWriter::new(&mut buf);
        writer.write_section(&clog, "Features", &section).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("#### Features"));
        assert!(output.contains("**parser:**"));
        assert!(output.contains("add feature"));
        assert!(output.contains("[abc12345]"));
    }

    #[test]
    fn write_section_empty() {
        let clog = test_clog();
        let section = BTreeMap::new();

        let mut buf = Vec::new();
        let mut writer = MarkdownWriter::new(&mut buf);
        writer.write_section(&clog, "Features", &section).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn write_section_with_closes() {
        let clog = test_clog();
        let commits = vec![Commit {
            hash: "abc1234567890abcdef1234567890abcdef123456".to_owned(),
            subject: "fix thing".to_owned(),
            component: "".to_owned(),
            closes: vec!["42".to_owned()],
            breaks: vec![],
            commit_type: "Bug Fixes".to_owned(),
        }];
        let component_key = "".to_owned();
        let mut section = BTreeMap::new();
        section.insert(&component_key, &commits);

        let mut buf = Vec::new();
        let mut writer = MarkdownWriter::new(&mut buf);
        writer.write_section(&clog, "Bug Fixes", &section).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("closes [#42]"));
    }
}
