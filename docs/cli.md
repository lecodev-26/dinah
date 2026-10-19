# DINAH command-line utility

Build or install the crate using the normal Cargo workflow, then invoke `dinah`.

```text
dinah --help
dinah --version
dinah init [PATH]
dinah run [PATH]
dinah --task build run [PATH]
dinah task [PATH]
dinah inspect [PATH]
dinah benchmark
dinah doctor [PATH]
dinah --format json inspect [PATH]
dinah --verbose doctor [PATH]
```

`PATH` defaults to the current directory. `init` creates `DINAH.toml` and refuses to overwrite it. The starter manifest contains a `[project]` section and an optional `[tasks]` section.

- `run` without `--task` validates the project name/version. To execute one declared task, use `dinah --task build run [PATH]`; the command is run by the platform shell with the project directory as its working directory. Its exit code is propagated. JSON mode captures stdout/stderr and reports them as escaped fields.
- `task` lists keys declared in `[tasks]`. Each value in `[tasks]` is an executable shell command string, for example `build = "cargo build"`; select it explicitly with `--task NAME` and `run`. Empty or whitespace-only values may be listed during manifest inspection, but execution rejects them with a validation error.
- `inspect` prints project metadata and the number of declared tasks.
- `benchmark` lists the seven benchmark suite names; it does not execute benchmarks. Run `cargo bench --bench <suite>` from the repository to launch a suite.
- `doctor` checks that the manifest is readable and the project name is present. It is a basic configuration check, not a security audit.
- `--format text|json` chooses human-readable or JSON output for command results. `--verbose` writes a diagnostic line to stderr.

The manifest reader intentionally supports only the documented starter subset; it is not a general TOML parser. It validates section-header syntax, rejects duplicate `[project]`/`[tasks]` tables and duplicate keys, requires quoted project metadata and task descriptions, rejects malformed assignments and invalid basic-string escapes, and reports manifest line numbers for syntax errors. Quoted strings decode the basic escapes `\"`, `\\`, `\b`, `\t`, `\n`, `\f`, `\r`, `\uXXXX`, and `\UXXXXXXXX`; Unicode escapes must represent valid Unicode scalar values. Whitespace inside the quotes is preserved exactly, including whitespace produced by escapes. Unknown sections are ignored after basic assignment syntax is checked. Unknown options, invalid formats and malformed project names produce actionable errors and a non-zero process exit. CLI tests cover argument parsing and invalid option combinations, manifest creation/no-overwrite, metadata/task extraction, malformed/adversarial manifests, duplicate declarations, JSON control escaping, selected task lookup, captured task output, propagated exit status, and end-to-end task, run, doctor, and benchmark output. The repository CI remains responsible for running the test suite.

## Build profiles and input limits

Use `cargo build` for the development (debug) profile and `cargo build --release` for the optimized profile. `cargo bench --bench <suite>` compiles and runs in release mode; `python3 scripts/run_benchmarks.py` runs all seven suites and records the operating system, architecture, Rust/Cargo versions, commit and timestamp in `benchmark-results/`. The CLI does not change the build profile at runtime.

The `DINAH.toml` reader accepts only the documented subset and rejects files larger than 1 MiB before reading them. Comments are recognized outside quoted values and JSON output escapes all control characters as valid sequences. This does not turn the parser into a complete TOML parser or replace a security audit.

## Task execution security

Strings declared in `[tasks]` are shell commands and may run programs, modify files or access the network with the user's permissions. `dinah run` does not execute them implicitly: a task must be explicitly selected with `--task NAME`. Review the manifest before running tasks from someone else's project; DINAH provides no sandbox, process isolation or interactive confirmation. In JSON mode, task output is captured and encoded as JSON fields; in text mode it is printed after the process finishes.


## Release status

The CLI is pre-alpha. The manifest parser is a deliberately limited starter subset, not a complete TOML implementation. Task commands are shell strings and run with the caller's permissions after explicit selection; there is no sandbox. Review the [v0.1.0 release checklist](release-checklist.md) before publishing or relying on this CLI for untrusted projects.
