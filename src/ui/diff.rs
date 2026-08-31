//! Helper types for file diffing.
//!
//! Uses [similar] for text diffs.

use similar::ChangeTag;

use crate::{state::path::FileContent, ui::theme::Stylize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HunkHeader {
    old_len: usize,
    old_start: usize,
    new_len: usize,
    new_start: usize,
}

impl HunkHeader {
    /// Creats a git-diff-style hunk header.
    ///
    /// E.g. `@@ -12,2 +12,2 @@`.
    fn from_group(group: &[similar::DiffOp]) -> Option<Self> {
        // This shouldn't happen, but we guard against it anyway.
        let (Some(first), Some(last)) = (group.first(), group.last()) else {
            return None;
        };

        let old_start = first.old_range().start;
        let new_start = first.new_range().start;
        Some(Self {
            old_start,
            old_len: last.old_range().end - old_start,
            new_start,
            new_len: last.new_range().end - new_start,
        })
    }

    /// Displays the hunk header in a git-diff-style.
    ///
    /// E.g. `@@ -12,2 +12,2 @@`.
    pub fn format(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start + 1,
            self.old_len,
            self.new_start + 1,
            self.new_len
        )
    }
}

/// A single line of a [Diff].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Add,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    header: Option<HunkHeader>,
    lines: Vec<DiffLine>,
}

impl Hunk {
    /// The total number of added + removed lines.
    pub fn changed_lines(&self) -> usize {
        self.lines
            .iter()
            .filter(|line| matches!(line.kind, DiffLineKind::Add | DiffLineKind::Remove))
            .count()
    }
}

/// A computed content diff for a single file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diff {
    Binary(String),
    Text(Vec<Hunk>),
    Identical,
}

impl Diff {
    /// Create a diff for a file that will change during a deploy.
    pub fn for_deploy(actual: &[u8], desired: &FileContent) -> Self {
        Self::build(
            actual,
            desired.bytes(),
            matches!(desired, FileContent::Binary(_)),
        )
    }

    /// Create a diff for a file that drifted since last deploy.
    pub fn for_drift(deployed: &FileContent, actual: &[u8]) -> Self {
        Self::build(
            deployed.bytes(),
            actual,
            matches!(deployed, FileContent::Binary(_)),
        )
    }

    /// Diff construction based on the content of two files.
    fn build(old: &[u8], new: &[u8], known_binary: bool) -> Self {
        if old == new {
            return Self::Identical;
        }

        if known_binary {
            return Self::binary_summary(old, new);
        }

        // If the text isn't valid UTF-8, fallback to a byte diff
        let (Ok(old_text), Ok(new_text)) = (std::str::from_utf8(old), std::str::from_utf8(new))
        else {
            return Self::binary_summary(old, new);
        };

        Self::text_diff(old_text, new_text)
    }

    /// In case of a binary file, we only show a single line of output, which presents
    /// the difference in file size.
    fn binary_summary(old: &[u8], new: &[u8]) -> Diff {
        Diff::Binary(format!(
            "binary · {} → {}",
            Self::human_bytes(old.len() as u64),
            Self::human_bytes(new.len() as u64)
        ))
    }

    /// Create a text diff with the [`similar`] crate.
    fn text_diff(old: &str, new: &str) -> Diff {
        let diff = similar::TextDiff::from_lines(old, new);
        let mut hunks = Vec::new();

        for group in diff.grouped_ops(3) {
            let header = HunkHeader::from_group(&group);
            let mut lines = Vec::new();

            for op in &group {
                for change in diff.iter_changes(op) {
                    let kind = match change.tag() {
                        ChangeTag::Equal => DiffLineKind::Context,
                        ChangeTag::Delete => DiffLineKind::Remove,
                        ChangeTag::Insert => DiffLineKind::Add,
                    };
                    let text = change.value().trim_end_matches('\n').to_string();
                    lines.push(DiffLine { kind, text });
                }
            }

            hunks.push(Hunk { header, lines })
        }

        Diff::Text(hunks)
    }

    /// A human readable byte size.
    // TODO: Library?
    pub fn human_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            return format!("{bytes} B");
        }

        let mut value = bytes as f64;
        for unit in ["KiB", "MiB", "GiB", "TiB"] {
            value /= 1024.0;
            if value < 1024.0 {
                return format!("{value:.1} {unit}");
            }
        }
        format!("{value:.1} PiB")
    }

    /// Render the diff as a git-style unified diff (without the file header),
    /// colored with the active palette.
    ///
    /// Lines are joined with `\n` without a trailing newline. An identical diff
    /// renders as an empty string.
    pub fn format(&self) -> String {
        match self {
            Diff::Identical => String::new(),
            Diff::Binary(summary) => summary.change().to_string(),
            Diff::Text(hunks) => {
                let mut lines = Vec::new();
                for hunk in hunks {
                    if let Some(header) = &hunk.header {
                        lines.push(header.format().change().to_string());
                    }
                    for line in &hunk.lines {
                        let styled = match line.kind {
                            DiffLineKind::Context => format!(" {}", line.text).unchanged(),
                            DiffLineKind::Add => format!("+{}", line.text).addition(),
                            DiffLineKind::Remove => format!("-{}", line.text).removal(),
                        };
                        lines.push(styled.to_string());
                    }
                }
                lines.join("\n")
            }
        }
    }
}
