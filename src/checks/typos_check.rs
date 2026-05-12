use std::path::Path;

use typos::Status;
use typos_cli::dict::BuiltIn;

use crate::diagnostic::{Diagnostic, Severity};

/// Source-code spell-check using the `typos` crate. The bundled dictionary
/// is curated specifically to keep false-positives low — typos can run
/// unattended on real codebases.
pub fn check(path: &Path, text: &str, out: &mut Vec<Diagnostic>) {
  let tokenizer = typos::tokens::Tokenizer::default();
  let dict = BuiltIn::new(Default::default());

  for typo in typos::check_str(text, &tokenizer, &dict) {
    let (line, column) = byte_offset_to_line_col(text, typo.byte_offset);

    let (message, suggestion) = match &typo.corrections {
      Status::Corrections(corrections) => {
        let suggestion = corrections
          .iter()
          .map(|c| c.as_ref())
          .collect::<Vec<_>>()
          .join(", ");
        (
          format!("`{}` should be `{}`", typo.typo, suggestion),
          Some(suggestion),
        )
      }
      Status::Invalid => (format!("`{}` is misspelled", typo.typo), None),
      // `Valid` should never reach us here, but be defensive.
      Status::Valid => continue,
    };

    out.push(Diagnostic {
      file: path.to_path_buf(),
      line,
      column,
      severity: Severity::Warning,
      rule: "typos",
      message,
      suggestion,
    });
  }
}

/// Convert a 0-based byte offset to 1-based (line, column).
fn byte_offset_to_line_col(text: &str, offset: usize) -> (usize, usize) {
  let offset = offset.min(text.len());
  let prefix = &text[..offset];
  let line = prefix.bytes().filter(|&b| b == b'\n').count() + 1;
  let line_start = prefix.rfind('\n').map_or(0, |p| p + 1);
  // Column is a 1-based byte offset within the line — good enough for
  // CLI use; we don't try to compute grapheme clusters.
  let column = offset - line_start + 1;
  (line, column)
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
  fn line_col_first_byte() {
    assert_eq!(byte_offset_to_line_col("hello", 0), (1, 1));
  }

  #[test]
  fn line_col_after_newline() {
    // "a\nbc" — offset 2 is 'b', start of line 2.
    assert_eq!(byte_offset_to_line_col("a\nbc", 2), (2, 1));
    // offset 3 is 'c'.
    assert_eq!(byte_offset_to_line_col("a\nbc", 3), (2, 2));
  }

  #[test]
  fn line_col_clamped_to_len() {
    let text = "abc";
    assert_eq!(byte_offset_to_line_col(text, 100), (1, 4));
  }

  #[test]
  fn detects_known_typo_with_suggestion() {
    // "teh" is in the canonical typos dictionary — one of the most
    // stable corrections, safe to depend on across crate versions.
    let diags = run("// teh quick brown fox\n");
    let typo = diags
      .iter()
      .find(|d| d.message.contains("teh"))
      .expect("expected `teh` to be flagged");
    assert_eq!(typo.rule, "typos");
    assert_eq!(typo.line, 1);
    assert_eq!(typo.suggestion.as_deref(), Some("the"));
  }

  #[test]
  fn clean_text_yields_no_typos() {
    assert!(run("fn main() {}\n").is_empty());
  }
}
