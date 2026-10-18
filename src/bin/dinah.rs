use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST: &str = "DINAH.toml";

fn usage() {
    println!("dinah {VERSION} — DINAH project utility\n\nUSAGE:\n  dinah [--help | --version]\n  dinah init [PATH]\n  dinah run [PATH]\n\nCOMMANDS:\n  init   Create a starter {MANIFEST} without overwriting existing files\n  run    Validate the manifest and report its configured project name\n\nPATH defaults to the current directory.");
}
fn project_path(argument: Option<&String>) -> io::Result<PathBuf> {
    match argument {
        Some(path) => Ok(PathBuf::from(path)),
        None => env::current_dir(),
    }
}
fn init(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    let manifest = path.join(MANIFEST);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&manifest)
        .map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "{} already exists; leaving it unchanged",
                        manifest.display()
                    ),
                )
            } else {
                error
            }
        })?;
    file.write_all(b"# DINAH project configuration\n[project]\nname = \"my-dinah-project\"\nversion = \"0.1.0\"\n")?;
    println!("Created {}", manifest.display());
    println!("Edit the [project] values before using this project.");
    Ok(())
}
fn run(path: &Path) -> io::Result<()> {
    let manifest = path.join(MANIFEST);
    let contents = fs::read_to_string(&manifest).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} not found; run `dinah init` first", manifest.display()),
            )
        } else {
            error
        }
    })?;
    let mut in_project_section = false;
    let mut project_name = None;
    for line in contents.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_project_section = line == "[project]";
            continue;
        }
        if in_project_section {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "name" {
                    let value = value.trim();
                    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
                        project_name = Some(value[1..value.len() - 1].trim().to_owned());
                    }
                }
            }
        }
    }
    let name = project_name
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} must define a non-empty quoted project.name",
                    manifest.display()
                ),
            )
        })?;
    println!("Project: {name}");
    println!("Manifest: {}", manifest.display());
    println!("Configuration validated. Task-file execution is not implemented yet.");
    Ok(())
}
fn execute(arguments: &[String]) -> io::Result<i32> {
    match arguments.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            usage();
            Ok(0)
        }
        Some("--version") | Some("-V") => {
            println!("dinah {VERSION}");
            Ok(0)
        }
        Some("init") => {
            if arguments.len() > 2 {
                usage();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "`dinah init` accepts at most one PATH",
                ));
            }
            init(&project_path(arguments.get(1))?)?;
            Ok(0)
        }
        Some("run") => {
            if arguments.len() > 2 {
                usage();
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "`dinah run` accepts at most one PATH",
                ));
            }
            run(&project_path(arguments.get(1))?)?;
            Ok(0)
        }
        Some(command) => {
            usage();
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown command `{command}`"),
            ))
        }
    }
}
fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match execute(&arguments) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("dinah: {error}");
            std::process::exit(2);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{execute, init, run};
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
    fn init_creates_manifest_and_refuses_to_overwrite() {
        let path = temp_dir();
        init(&path).unwrap();
        let manifest = path.join("DINAH.toml");
        let original = fs::read_to_string(&manifest).unwrap();
        assert!(original.contains("name = \"my-dinah-project\""));
        assert!(init(&path).is_err());
        assert_eq!(fs::read_to_string(&manifest).unwrap(), original);
        fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn run_validates_project_name() {
        let path = temp_dir();
        fs::write(
            path.join("DINAH.toml"),
            "[project]\nname = \"sample\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert!(run(&path).is_ok());
        fs::write(path.join("DINAH.toml"), "[project]\nversion = \"0.1.0\"\n").unwrap();
        assert!(run(&path).is_err());
        fs::remove_dir_all(path).unwrap();
    }
}
