# FastLint

FastLint checks all your files in any programming language,
making vibe coding safer and faster.

Rather than reimplementing analyses, it bundles best-in-class
open-source checkers from crates.io.

## Install

From [crates.io](https://crates.io/crates/fastlint):

```sh
cargo install fastlint
```

From a local checkout:

```sh
cargo install --path .
```

## Usage

```sh
fastlint                  # check the current directory
fastlint source/frontend  # check a specific directory or file
fastlint --json           # emit diagnostics as a JSON array
```

`.gitignore`, `.ignore`, hidden files, and obvious binary/lock files
are skipped automatically.

## Checks

| Rule                  | Severity | Source                                        |
| --------------------- | -------- | --------------------------------------------- |
| `typos`               | warning  | [`typos`](https://crates.io/crates/typos) source-code spell-checker |
| `merge-conflict`      | error    | unresolved `<<<<<<<` / `=======` / `>>>>>>>` markers |
| `trailing-whitespace` | warning  | trailing spaces/tabs at end of line           |

## Exit codes

| Code | Meaning                                  |
| ---- | ---------------------------------------- |
| `0`  | no errors (warnings allowed)             |
| `1`  | one or more error diagnostics            |
| `2`  | usage error (e.g. path does not exist)   |
