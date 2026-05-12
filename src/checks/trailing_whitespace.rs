use std::path::Path;

use crate::diagnostic::{Diagnostic, Severity};

/// Warn on trailing whitespace at the end of any line.
pub fn check(path: &Path, text: &str, out: &mut Vec<Diagnostic>) {
  for (idx, line) in text.lines().enumerate() {
    let trimmed = line.trim_end_matches(&[' ', '\t'][..]);
    if trimmed.len() == line.len() {
      continue;
    }
    out.push(Diagnostic {
      file: path.to_path_buf(),
      line: idx + 1,
      column: trimmed.len() + 1,
      severity: Severity::Warning,
      rule: "trailing-whitespace",
      message: "trailing whitespace".to_string(),
      suggestion: None,
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn run(text: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    check(&PathBuf::from("test.rs"), text, &mut out);
    out
  }

  #[test]
  fn flags_trailing_spaces() {
    let diags = run("ok\nbad   \nfine\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, 2);
    // 1-based column pointing at the first whitespace char (after "bad").
    assert_eq!(diags[0].column, 4);
    assert_eq!(diags[0].severity, Severity::Warning);
  }

  #[test]
  fn flags_trailing_tabs() {
    let diags = run("text\t\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].column, 5);
  }

  #[test]
  fn empty_line_is_not_flagged() {
    // Truly empty lines have nothing to trim — only lines containing
    // pure trailing whitespace are problematic, and `\n\n` produces
    // an empty middle line, not a whitespace line.
    let diags = run("a\n\nb\n");
    assert!(diags.is_empty(), "got {diags:?}");
  }

  #[test]
  fn whitespace_only_line_is_flagged() {
    let diags = run("a\n   \nb\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].line, 2);
    assert_eq!(diags[0].column, 1);
  }
}
