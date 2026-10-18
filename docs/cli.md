# DINAH command-line utility

Build or install the crate using the normal Cargo workflow, then invoke `dinah`.

```text
dinah --help
dinah --version
dinah init [PATH]
dinah run [PATH]
dinah task [PATH]
dinah inspect [PATH]
dinah benchmark
dinah doctor [PATH]
dinah --format json inspect [PATH]
dinah --verbose doctor [PATH]
```

`PATH` defaults to the current directory. `init` creates `DINAH.toml` and refuses to overwrite it. The starter manifest contains a `[project]` section and an optional `[tasks]` section.

- `run` validates the project name/version and reports the project. It does **not** execute tasks yet.
- `task` lists keys declared in `[tasks]`; values are currently descriptive strings, not commands to execute.
- `inspect` prints project metadata and the number of declared tasks.
- `benchmark` lists the seven benchmark suite names; it does not execute benchmarks. Run `cargo bench --bench <suite>` from the repository to launch a suite.
- `doctor` checks that the manifest is readable and the project name is present. It is a basic configuration check, not a security audit.
- `--format text|json` chooses human-readable or JSON output for command results. `--verbose` writes a diagnostic line to stderr.

The manifest reader intentionally supports only the starter subset needed by these commands. It is not a general TOML parser. Unknown options, invalid formats and malformed project names produce actionable errors and a non-zero process exit. CLI unit tests cover argument parsing, manifest creation/no-overwrite, metadata/task extraction and invalid manifests; the repository CI remains responsible for running the test suite.
