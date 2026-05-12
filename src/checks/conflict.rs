use std::path::Path;

use crate::diagnostic::{Diagnostic, Severity};

/// Detect unresolved Git merge-conflict markers.
pub fn check(path: &Path, text: &str, out: &mut Vec<Diagnostic>) {
  for (idx, line) in text.lines().enumerate() {
    let marker = if line.starts_with("<<<<<<< ") || line == "<<<<<<<" {
      Some("start")
    } else if line == "=======" {
      Some("separator")
    } else if line.starts_with(">>>>>>> ") || line == ">>>>>>>" {
      Some("end")
    } else {
      None
    };

    if let Some(kind) = marker {
      out.push(Diagnostic {
        file: path.to_path_buf(),
        line: idx + 1,
        column: 1,
        severity: Severity::Error,
        rule: "merge-conflict",
        message: format!("unresolved merge-conflict marker ({kind})"),
        suggestion: None,
      });
    }
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
  fn detects_all_three_marker_kinds() {
    let text = "fn ok() {}\n<<<<<<< HEAD\nlhs\n=======\nrhs\n>>>>>>> feature\n";
    let diags = run(text);
    assert_eq!(diags.len(), 3);
    assert_eq!(diags[0].line, 2);
    assert!(diags[0].message.contains("start"));
    assert_eq!(diags[1].line, 4);
    assert!(diags[1].message.contains("separator"));
    assert_eq!(diags[2].line, 6);
    assert!(diags[2].message.contains("end"));
    for d in &diags {
      assert_eq!(d.severity, Severity::Error);
      assert_eq!(d.rule, "merge-conflict");
    }
  }

  #[test]
  fn accepts_bare_markers_without_branch_name() {
    let diags = run("<<<<<<<\n=======\n>>>>>>>\n");
    assert_eq!(diags.len(), 3);
  }

  #[test]
  fn ignores_similar_but_unrelated_lines() {
    // Same chars but wrong count / context — must not fire.
    let diags = run(
      "<<<< not a marker\n\
             ====== six equals only\n\
             >>>>>>>>> nine angles\n\
             // ======= inside comment\n",
    );
    assert!(diags.is_empty(), "got {diags:?}");
  }

  #[test]
  fn clean_file_is_clean() {
    assert!(run("fn main() {}\n").is_empty());
  }
}
