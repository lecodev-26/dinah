use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST: &str = "DINAH.toml";
const BENCHMARKS: &[&str] = &[
    "runtime",
    "memory",
    "compute",
    "ecs",
    "simulation",
    "network",
    "storage",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug)]
struct CliOptions {
    command: String,
    path: Option<String>,
    format: OutputFormat,
    verbose: bool,
}

fn usage() {
    println!("dinah {VERSION} — DINAH project utility\n\nUSAGE:\n  dinah [--help | --version]\n  dinah [--format text|json] [--verbose] <COMMAND> [PATH]\n\nCOMMANDS:\n  init       Create DINAH.toml without overwriting\n  run        Validate the manifest (does not execute tasks yet)\n  task       List task declarations in [tasks]\n  inspect    Show project metadata\n  benchmark  List available benchmark suites\n  doctor     Check basic project health\n\nOPTIONS:\n  --format text|json  Select output format (default: text)\n  --verbose           Print diagnostic logging to stderr\n  -h, --help          Show help\n  -V, --version       Show version\n\nPATH defaults to the current directory. The manifest parser supports only the documented starter subset, not full TOML.");
}

fn json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push(' '),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn emit(format: OutputFormat, text: &str, fields: &[(&str, String)]) {
    match format {
        OutputFormat::Text => println!("{text}"),
        OutputFormat::Json => {
            let body = fields
                .iter()
                .map(|(key, value)| format!("{}:{}", json_string(key), json_string(value)))
                .collect::<Vec<_>>()
                .join(",");
            println!("{{{body}}}");
        }
    }
}

fn parse_options(arguments: &[String]) -> io::Result<Option<CliOptions>> {
    if arguments.is_empty() || arguments.iter().any(|arg| arg == "--help" || arg == "-h") {
        usage();
        return Ok(None);
    }
    if arguments
        .iter()
        .any(|arg| arg == "--version" || arg == "-V")
    {
        println!("dinah {VERSION}");
        return Ok(None);
    }
    let mut command = None;
    let mut path = None;
    let mut format = OutputFormat::Text;
    let mut verbose = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--format" => {
                index += 1;
                format = match arguments.get(index).map(String::as_str) {
                    Some("text") => OutputFormat::Text,
                    Some("json") => OutputFormat::Json,
                    Some(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "invalid --format value; choose text or json",
                        ))
                    }
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "--format requires text or json",
                        ))
                    }
                };
            }
            "--verbose" => verbose = true,
            value if value.starts_with('-') => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option {value}"),
                ))
            }
            value if command.is_none() => command = Some(value.to_owned()),
            value if path.is_none() => path = Some(value.to_owned()),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "too many positional arguments; expected COMMAND [PATH]",
                ))
            }
        }
        index += 1;
    }
    let command = command.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing command; run dinah --help",
        )
    })?;
    if !matches!(
        command.as_str(),
        "init" | "run" | "task" | "inspect" | "benchmark" | "doctor"
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown command {command}; run dinah --help"),
        ));
    }
    Ok(Some(CliOptions {
        command,
        path,
        format,
        verbose,
    }))
}

fn project_path(argument: Option<&String>) -> io::Result<PathBuf> {
    match argument {
        Some(path) => Ok(PathBuf::from(path)),
        None => env::current_dir(),
    }
}

fn init(path: &Path, format: OutputFormat) -> io::Result<()> {
    fs::create_dir_all(path).map_err(|e| friendly_io(e, "create project directory"))?;
    let manifest = path.join(MANIFEST);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest)
        .map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{} already exists; it was not changed", manifest.display()),
                )
            } else {
                friendly_io(error, "create project manifest")
            }
        })?;
    file.write_all(b"# DINAH project configuration\n[project]\nname = \"my-dinah-project\"\nversion = \"0.1.0\"\n\n[tasks]\n# build = \"Describe the task here\"\n")
        .map_err(|e| friendly_io(e, "write project manifest"))?;
    emit(
        format,
        &format!("Created {}", manifest.display()),
        &[
            ("status", "created".into()),
            ("manifest", manifest.display().to_string()),
        ],
    );
    Ok(())
}

fn friendly_io(error: io::Error, action: &str) -> io::Error {
    let message = match error.kind() {
        io::ErrorKind::NotFound => {
            format!("{action} failed: path not found; check the PATH or run dinah init")
        }
        io::ErrorKind::PermissionDenied => {
            format!("{action} failed: permission denied; check filesystem permissions")
        }
        io::ErrorKind::AlreadyExists => {
            format!("{action} failed: the file already exists; DINAH will not overwrite it")
        }
        io::ErrorKind::InvalidData => {
            format!("{action} failed: invalid project configuration; check [project].name")
        }
        _ => format!("{action} failed: {error}"),
    };
    io::Error::new(error.kind(), message)
}

fn read_manifest(path: &Path) -> io::Result<(String, String, Vec<String>)> {
    let manifest = path.join(MANIFEST);
    let contents = fs::read_to_string(&manifest).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} not found; run dinah init first", manifest.display()),
            )
        } else {
            friendly_io(error, "read project manifest")
        }
    })?;
    let mut section = String::new();
    let mut name = None;
    let mut version = None;
    let mut tasks = Vec::new();
    for source_line in contents.lines() {
        let line = source_line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_owned();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if section == "project" && key == "name" {
                name = parse_quoted(value);
            } else if section == "project" && key == "version" {
                version = parse_quoted(value);
            } else if section == "tasks" && !key.is_empty() {
                tasks.push(key.to_owned());
            }
        }
    }
    let name = name.filter(|value| !value.is_empty()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{} must define a non-empty quoted project.name",
                manifest.display()
            ),
        )
    })?;
    Ok((name, version.unwrap_or_else(|| "unspecified".into()), tasks))
}

fn parse_quoted(value: &str) -> Option<String> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(value[1..value.len() - 1].trim().to_owned())
    } else {
        None
    }
}

fn run(path: &Path, format: OutputFormat) -> io::Result<()> {
    let (name, version, _) = read_manifest(path)?;
    emit(
        format,
        &format!("Project {name} ({version}) validated. Task execution is not implemented yet."),
        &[
            ("status", "validated".into()),
            ("name", name),
            ("version", version),
            ("task_execution", "not_implemented".into()),
        ],
    );
    Ok(())
}

fn task(path: &Path, format: OutputFormat) -> io::Result<()> {
    let (_, _, tasks) = read_manifest(path)?;
    if format == OutputFormat::Json {
        let values = tasks
            .iter()
            .map(|task| json_string(task))
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"tasks\":[{values}],\"execution\":\"not_implemented\"}}");
    } else if tasks.is_empty() {
        println!("No tasks declared in [tasks]. Add task names to DINAH.toml.");
    } else {
        println!("Declared tasks (not executable yet):");
        for name in tasks {
            println!("  - {name}");
        }
    }
    Ok(())
}

fn inspect(path: &Path, format: OutputFormat) -> io::Result<()> {
    let (name, version, tasks) = read_manifest(path)?;
    let manifest = path.join(MANIFEST);
    emit(
        format,
        &format!(
            "Project: {name}\nVersion: {version}\nManifest: {}\nDeclared tasks: {}",
            manifest.display(),
            tasks.len()
        ),
        &[
            ("name", name),
            ("version", version),
            ("manifest", manifest.display().to_string()),
            ("declared_tasks", tasks.len().to_string()),
        ],
    );
    Ok(())
}

fn benchmark(format: OutputFormat) {
    if format == OutputFormat::Json {
        let values = BENCHMARKS
            .iter()
            .map(|name| json_string(name))
            .collect::<Vec<_>>()
            .join(",");
        println!("{{\"suites\":[{values}],\"execution\":\"use cargo bench --bench <suite>\",\"status\":\"listed_only\"}}");
    } else {
        println!("Available benchmark suites (this command only lists them):");
        for suite in BENCHMARKS {
            println!("  - {suite}");
        }
        println!("Run a suite with cargo bench --bench <suite> in the repository.");
    }
}

fn doctor(path: &Path, format: OutputFormat) -> io::Result<()> {
    let manifest = path.join(MANIFEST);
    let directory_ok = path.is_dir();
    let (name, version, _) = read_manifest(path)?;
    let checks = [
        ("project_directory", directory_ok),
        ("manifest_readable", manifest.is_file()),
        ("project_name", !name.trim().is_empty()),
    ];
    let passed = checks.iter().filter(|(_, ok)| *ok).count();
    let status = if passed == checks.len() {
        "healthy"
    } else {
        "warning"
    };
    if format == OutputFormat::Json {
        println!(
            "{{\"status\":{},\"name\":{},\"version\":{},\"checks\":{{{}}}}}",
            json_string(status),
            json_string(&name),
            json_string(&version),
            checks
                .iter()
                .map(|(key, ok)| format!("{}:{}", json_string(key), ok))
                .collect::<Vec<_>>()
                .join(",")
        );
    } else {
        println!("DINAH doctor: {status}");
        for (key, ok) in checks {
            println!("  [{}] {key}", if ok { "ok" } else { "fail" });
        }
        println!("  project: {name} ({version})");
        println!("Note: this is a basic manifest check, not a security audit.");
    }
    Ok(())
}

fn execute(arguments: &[String]) -> io::Result<i32> {
    let Some(options) = parse_options(arguments)? else {
        return Ok(0);
    };
    if options.verbose {
        eprintln!(
            "dinah: command={} format={:?}",
            options.command, options.format
        );
    }
    match options.command.as_str() {
        "init" => init(&project_path(options.path.as_ref())?, options.format)?,
        "run" => run(&project_path(options.path.as_ref())?, options.format)?,
        "task" => task(&project_path(options.path.as_ref())?, options.format)?,
        "inspect" => inspect(&project_path(options.path.as_ref())?, options.format)?,
        "benchmark" => benchmark(options.format),
        "doctor" => doctor(&project_path(options.path.as_ref())?, options.format)?,
        _ => unreachable!("command validated by parse_options"),
    }
    Ok(0)
}

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match execute(&arguments) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("dinah: {error}");
            eprintln!("Try dinah --help for usage.");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{execute, init, parse_options, read_manifest, run, OutputFormat};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("dinah-cli-test-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn help_and_version_are_successful() {
        assert_eq!(execute(&["--help".into()]).unwrap(), 0);
        assert_eq!(execute(&["--version".into()]).unwrap(), 0);
    }

    #[test]
    fn parses_format_and_verbose_options() {
        let parsed = parse_options(&[
            "--format".into(),
            "json".into(),
            "--verbose".into(),
            "benchmark".into(),
        ])
        .unwrap()
        .unwrap();
        assert_eq!(parsed.format, OutputFormat::Json);
        assert!(parsed.verbose);
        assert!(parse_options(&["--format".into(), "xml".into(), "doctor".into()]).is_err());
    }

    #[test]
    fn init_creates_manifest_and_refuses_to_overwrite() {
        let path = temp_dir();
        init(&path, OutputFormat::Text).unwrap();
        let manifest = path.join("DINAH.toml");
        let original = fs::read_to_string(&manifest).unwrap();
        assert!(original.contains("name = \"my-dinah-project\""));
        assert!(init(&path, OutputFormat::Text).is_err());
        assert_eq!(fs::read_to_string(&manifest).unwrap(), original);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn reads_project_metadata_and_task_names() {
        let path = temp_dir();
        fs::write(path.join("DINAH.toml"), "[project]\nname = \"sample\"\nversion = \"0.2.0\"\n[tasks]\nbuild = \"description only\"\ntest = \"description only\"\n").unwrap();
        let (name, version, tasks) = read_manifest(&path).unwrap();
        assert_eq!(name, "sample");
        assert_eq!(version, "0.2.0");
        assert_eq!(tasks, vec!["build", "test"]);
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn run_rejects_missing_project_name() {
        let path = temp_dir();
        fs::write(path.join("DINAH.toml"), "[project]\nversion = \"0.1.0\"\n").unwrap();
        assert!(run(&path, OutputFormat::Text).is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
