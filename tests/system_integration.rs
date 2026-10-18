//! Public integration checks for Runtime/Network, Native and the CLI surface.
use dinah::native::{Hal, HostPlatform, Platform};
use dinah::network::runtime::NetworkWorkQueue;
use dinah::observability::{MetricsRegistry, Subsystem};
use dinah::runtime::Scheduler;
use std::num::NonZeroUsize;
use std::process::Command;
use std::time::{Duration, Instant};

#[test]
fn runtime_dispatches_network_work_and_reports_result_explicitly() {
    let scheduler = Scheduler::new();
    let mut queue = NetworkWorkQueue::<&'static str>::new(NonZeroUsize::new(2).unwrap());
    let delivered = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let task_output = std::sync::Arc::clone(&delivered);
    assert!(queue
        .enqueue(Box::new(move || {
            task_output.lock().unwrap().push(b"packet-1".to_vec());
            Ok(())
        }))
        .is_ok());

    assert_eq!(queue.len(), 1, "enqueue must not execute network work");
    assert_eq!(queue.dispatch_next(&scheduler), Some(Ok(Ok(()))));
    assert!(queue.is_empty());
    assert_eq!(*delivered.lock().unwrap(), vec![b"packet-1".to_vec()]);

    let metrics = MetricsRegistry::new();
    metrics
        .increment(Subsystem::Network, "work.dispatched", 1)
        .unwrap();
    assert!(metrics.export_json().unwrap().contains("work.dispatched"));
}

#[test]
fn native_platform_metadata_can_be_consumed_by_scheduler() {
    let platform = HostPlatform::new();
    let scheduler = Scheduler::new();
    let result = scheduler
        .schedule(|| {
            Ok::<_, &'static str>((
                platform.operating_system(),
                platform.architecture(),
                platform.process_id(),
                platform.is_unix(),
            ))
        })
        .unwrap()
        .unwrap();
    assert!(!result.0.is_empty());
    assert!(!result.1.is_empty());
    assert!(result.2 > 0);
    assert_eq!(result.3, cfg!(unix));
    assert!(
        platform.elapsed_since(Instant::now() - Duration::from_millis(1))
            >= Duration::from_millis(1)
    );
}

#[test]
fn cli_binary_initializes_and_inspects_project_as_json_without_overwrite() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("dinah-system-cli-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();

    let init = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .arg("init")
        .arg(&root)
        .output()
        .expect("CLI starts");
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(root.join("DINAH.toml").is_file());

    let inspect = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "inspect"])
        .arg(&root)
        .output()
        .expect("CLI starts");
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let stdout = String::from_utf8(inspect.stdout).unwrap();
    assert!(stdout.contains("my-dinah-project"), "{stdout}");

    let original = std::fs::read(root.join("DINAH.toml")).unwrap();
    let second_init = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .arg("init")
        .arg(&root)
        .output()
        .expect("CLI starts");
    assert!(
        !second_init.status.success(),
        "init must refuse to overwrite"
    );
    assert_eq!(std::fs::read(root.join("DINAH.toml")).unwrap(), original);
    let manifest = root.join("DINAH.toml");
    std::fs::write(
        &manifest,
        "[project]\nname = \"integration#project\" # trailing comment\nversion = \"0.3.0\"\n\n[tasks]\nbuild = \"description only\"\ntest = \"description only\"\n",
    )
    .unwrap();

    let task = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "task"])
        .arg(&root)
        .output()
        .expect("task CLI starts");
    assert!(
        task.status.success(),
        "{}",
        String::from_utf8_lossy(&task.stderr)
    );
    let task_json = String::from_utf8(task.stdout).unwrap();
    assert!(
        task_json.contains("\"tasks\":[\"build\",\"test\"]"),
        "{task_json}"
    );
    assert!(task_json.contains("explicit_task_selection"), "{task_json}");

    let run = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "run"])
        .arg(&root)
        .output()
        .expect("run CLI starts");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let run_json = String::from_utf8(run.stdout).unwrap();
    assert!(run_json.contains("integration#project"), "{run_json}");
    assert!(
        run_json.contains("\"task_execution\":\"explicit_opt_in\""),
        "{run_json}"
    );

    let doctor = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "doctor"])
        .arg(&root)
        .output()
        .expect("doctor CLI starts");
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let doctor_json = String::from_utf8(doctor.stdout).unwrap();
    assert!(
        doctor_json.contains("\"status\":\"healthy\""),
        "{doctor_json}"
    );
    assert!(
        doctor_json.contains("\"manifest_readable\":true"),
        "{doctor_json}"
    );

    let benchmark = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "benchmark"])
        .output()
        .expect("benchmark CLI starts");
    assert!(
        benchmark.status.success(),
        "{}",
        String::from_utf8_lossy(&benchmark.stderr)
    );
    let benchmark_json = String::from_utf8(benchmark.stdout).unwrap();
    assert!(
        benchmark_json.contains("\"status\":\"listed_only\""),
        "{benchmark_json}"
    );
    assert!(benchmark_json.contains("\"runtime\""), "{benchmark_json}");

    let task_manifest = "[project]\nname = \"integration-task\"\n[tasks]\npass = \"printf task-ok\"\nunicode = \"printf \\u263A\"\nfail = \"exit 7\"\n";
    std::fs::write(&manifest, task_manifest).unwrap();
    let executed = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "--task", "pass", "run"])
        .arg(&root)
        .output()
        .expect("task execution starts");
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let executed_json = String::from_utf8(executed.stdout).unwrap();
    assert!(
        executed_json.contains("\"stdout\":\"task-ok\""),
        "{executed_json}"
    );
    let unicode = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "--task", "unicode", "run"])
        .arg(&root)
        .output()
        .expect("Unicode-escaped task starts");
    assert!(
        unicode.status.success(),
        "{}",
        String::from_utf8_lossy(&unicode.stderr)
    );
    let unicode_json = String::from_utf8(unicode.stdout).unwrap();
    assert!(unicode_json.contains("\"stdout\":\"☺\""), "{unicode_json}");
    let failed = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--task", "fail", "run"])
        .arg(&root)
        .status()
        .expect("failing task starts");
    assert_eq!(failed.code(), Some(7));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cli_rejects_invalid_unicode_and_unknown_tasks_without_running_commands() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "dinah-cli-negative-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let manifest = root.join("DINAH.toml");

    std::fs::write(
        &manifest,
        r#"[project]
name = "invalid-unicode"
[tasks]
invalid = "printf \uD800"
"#,
    )
    .unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "run"])
        .arg(&root)
        .output()
        .expect("invalid manifest check starts");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr)
        .contains("task descriptions must be quoted strings"));

    std::fs::write(
        &manifest,
        r#"[project]
name = "negative-test"
[tasks]
fail = "exit 7"
"#,
    )
    .unwrap();
    let unknown = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--task", "missing", "run"])
        .arg(&root)
        .output()
        .expect("unknown task check starts");
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("task 'missing' is not declared"));

    let failed_json = Command::new(env!("CARGO_BIN_EXE_dinah"))
        .args(["--format", "json", "--task", "fail", "run"])
        .arg(&root)
        .output()
        .expect("JSON task execution starts");
    assert_eq!(failed_json.status.code(), Some(7));
    let body = String::from_utf8(failed_json.stdout).unwrap();
    assert!(body.contains(r#""status":"failure""#), "{body}");
    assert!(body.contains(r#""exit_code":7"#), "{body}");

    std::fs::remove_dir_all(root).unwrap();
}
