use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::diagnostic::Diagnostic;

mod conflict;
mod trailing_whitespace;
mod typos_check;

/// Run every check against every file, in parallel, and gather diagnostics.
pub fn run(files: &[PathBuf]) -> Vec<Diagnostic> {
  let mut diagnostics: Vec<Diagnostic> = files
    .par_iter()
    .flat_map_iter(|path| check_file(path))
    .collect();

  diagnostics.sort_by(|a, b| {
    a.file
      .cmp(&b.file)
      .then(a.line.cmp(&b.line))
      .then(a.column.cmp(&b.column))
      .then(a.rule.cmp(b.rule))
  });

  diagnostics
}

fn check_file(path: &Path) -> Vec<Diagnostic> {
  let bytes = match std::fs::read(path) {
    Ok(bytes) => bytes,
    // Unreadable files aren't fatal; just skip them.
    Err(_) => return Vec::new(),
  };

  // Heuristic: a NUL byte in the first 8 KiB means binary — skip.
  let probe = &bytes[..bytes.len().min(8192)];
  if probe.contains(&0) {
    return Vec::new();
  }

  let text = match std::str::from_utf8(&bytes) {
    Ok(text) => text,
    Err(_) => return Vec::new(),
  };

  let mut out = Vec::new();
  conflict::check(path, text, &mut out);
  trailing_whitespace::check(path, text, &mut out);
  typos_check::check(path, text, &mut out);
  out
}
