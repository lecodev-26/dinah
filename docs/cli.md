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

The manifest reader intentionally supports only the documented starter subset; it is not a general TOML parser. It validates section-header syntax, rejects duplicate `[project]`/`[tasks]` tables and duplicate keys, requires quoted project metadata and task descriptions, rejects malformed assignments and invalid basic-string escapes, and reports manifest line numbers for syntax errors. Unknown sections are ignored after basic assignment syntax is checked. Unknown options, invalid formats and malformed project names produce actionable errors and a non-zero process exit. CLI tests cover argument parsing, manifest creation/no-overwrite, metadata/task extraction, malformed/adversarial manifests, duplicate declarations, JSON control escaping, and end-to-end task, run, doctor, and benchmark output. The repository CI remains responsible for running the test suite.

## Perfiles de compilación y límites de entrada

Usa `cargo build` para el perfil de desarrollo (debug) y `cargo build --release` para el perfil optimizado. `cargo bench --bench <suite>` compila y ejecuta en modo release; `python3 scripts/run_benchmarks.py` ejecuta las siete suites y registra el sistema operativo, arquitectura, versiones de Rust/Cargo, commit y marca temporal en `benchmark-results/`. El CLI no cambia el perfil de compilación en tiempo de ejecución.

El lector de `DINAH.toml` acepta únicamente el subconjunto documentado y rechaza archivos mayores de 1 MiB antes de leerlos. Los comentarios se reconocen fuera de valores entrecomillados y la salida JSON escapa todos los controles como secuencias válidas. Esto no convierte el parser en un parser TOML completo ni sustituye una auditoría de seguridad.
