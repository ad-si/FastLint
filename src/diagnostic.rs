use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
  Error,
  Warning,
}

impl Severity {
  pub fn label(self) -> &'static str {
    match self {
      Severity::Error => "error",
      Severity::Warning => "warning",
    }
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
  pub file: PathBuf,
  pub line: usize,
  pub column: usize,
  pub severity: Severity,
  /// Stable rule identifier, e.g. `merge-conflict`, `typos`, `trailing-whitespace`.
  pub rule: &'static str,
  pub message: String,
  /// Optional suggested fix or correction.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suggestion: Option<String>,
}
