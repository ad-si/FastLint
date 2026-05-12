use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

mod checks;
mod diagnostic;
mod output;
mod walker;

use diagnostic::Severity;

/// FastLint — checks all your files in any programming language,
/// making vibe coding safer and faster.
#[derive(Debug, Parser)]
#[command(name = "fastlint", version, about, long_about = None)]
struct Cli {
  /// Paths to check. Defaults to the current directory.
  #[arg(value_name = "PATH")]
  paths: Vec<PathBuf>,

  /// Emit diagnostics as a JSON array on stdout.
  #[arg(long)]
  json: bool,

  /// Disable ANSI colors in human output.
  #[arg(long)]
  no_color: bool,
}

fn main() -> ExitCode {
  let cli = Cli::parse();

  let paths = if cli.paths.is_empty() {
    vec![PathBuf::from(".")]
  } else {
    cli.paths.clone()
  };

  let files = match walker::collect_files(&paths) {
    Ok(files) => files,
    Err(err) => {
      eprintln!("fastlint: {err:#}");
      return ExitCode::from(2);
    }
  };

  let diagnostics = checks::run(&files);

  if cli.json {
    output::print_json(&diagnostics);
  } else {
    output::print_human(&diagnostics, !cli.no_color);
  }

  let has_errors = diagnostics.iter().any(|d| d.severity == Severity::Error);
  if has_errors {
    ExitCode::from(1)
  } else {
    ExitCode::SUCCESS
  }
}
