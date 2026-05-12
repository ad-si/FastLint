use std::io::{self, Write};

use crate::diagnostic::{Diagnostic, Severity};

pub fn print_json(diagnostics: &[Diagnostic]) {
  let stdout = io::stdout();
  let mut out = stdout.lock();
  // Always emit a JSON array — even if empty — so consumers can
  // safely parse without special-casing.
  let _ = serde_json::to_writer(&mut out, diagnostics);
  let _ = out.write_all(b"\n");
}

pub fn print_human(diagnostics: &[Diagnostic], color: bool) {
  let stdout = io::stdout();
  let mut out = stdout.lock();

  if diagnostics.is_empty() {
    let _ = writeln!(out, "{}", paint("No issues found.", Color::Green, color));
    return;
  }

  let mut errors = 0usize;
  let mut warnings = 0usize;

  for d in diagnostics {
    match d.severity {
      Severity::Error => errors += 1,
      Severity::Warning => warnings += 1,
    }

    let sev_color = match d.severity {
      Severity::Error => Color::Red,
      Severity::Warning => Color::Yellow,
    };

    let location = format!("{}:{}:{}", d.file.display(), d.line, d.column);
    let _ = writeln!(
      out,
      "{} {} {} {}",
      paint(&location, Color::Cyan, color),
      paint(d.severity.label(), sev_color, color),
      paint(&format!("[{}]", d.rule), Color::Dim, color),
      d.message,
    );

    if let Some(sugg) = &d.suggestion {
      let _ =
        writeln!(out, "  {} {}", paint("help:", Color::Green, color), sugg,);
    }
  }

  let _ = writeln!(out);
  let _ = writeln!(
    out,
    "{} {} error(s), {} warning(s)",
    paint("summary:", Color::Bold, color),
    errors,
    warnings,
  );
}

#[derive(Clone, Copy)]
enum Color {
  Red,
  Yellow,
  Green,
  Cyan,
  Bold,
  Dim,
}

fn paint(text: &str, color: Color, enabled: bool) -> String {
  if !enabled {
    return text.to_string();
  }
  let code = match color {
    Color::Red => "\x1b[31m",
    Color::Yellow => "\x1b[33m",
    Color::Green => "\x1b[32m",
    Color::Cyan => "\x1b[36m",
    Color::Bold => "\x1b[1m",
    Color::Dim => "\x1b[2m",
  };
  format!("{code}{text}\x1b[0m")
}
