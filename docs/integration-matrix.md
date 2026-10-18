# DINAH integration matrix (v0.1 work in progress)

This records executable integration evidence without implying automatic wiring between all subsystems.

| Boundary | Executable evidence | Guarantee / limitation |
|---|---|---|
| Runtime → Network | `tests/network_advanced.rs`, `tests/system_integration.rs` | Caller-created one-shot work dispatches synchronously through `Scheduler`; enqueue alone does not execute it and no background thread/socket is created. |
| Runtime → Native | `tests/system_integration.rs` | Host OS, architecture, process ID and monotonic elapsed time can be consumed inside scheduled work; this is host metadata, not direct hardware access. |
| CLI → manifest/tasks | `tests/system_integration.rs` invokes the actual `dinah` binary | `init` creates a manifest and refuses overwrite; `inspect --format json` reads it; `run` validates by default, while `--task NAME run` explicitly executes a declared shell command and propagates its exit status. Task execution is unsandboxed. |
| TCP → Simulation → Runtime/Storage | `tests/system_pipeline.rs` | A loopback `Endpoint`/`Connection` transfers one fixed-width value over the implemented TCP byte stream; the caller explicitly applies it during a logical Simulation step and persists/reloads the result through `StorageWorkQueue` and `DiskStore`. The test does not imply framing, authentication, or server authority. |
| Network prediction → Simulation/ECS → Storage | `tests/system_pipeline.rs` | A bounded prediction history reconciles an authoritative snapshot and replays pending input in order before the caller transfers corrected state into ECS/Simulation. A separate scenario exercises tick-delayed `NetworkSimulator` delivery, applies the received input explicitly to Simulation, and persists/reloads the result through Runtime and `DiskStore`; this is deterministic simulated delivery, not a real network protocol or server authority. |
| Storage and application data | `tests/system_pipeline.rs` | Explicit FIFO storage work writes a simulation snapshot to `MemoryStore`; a separate cross-subsystem test dispatches PUT/DELETE through Runtime into `DiskStore` and verifies the surviving snapshot after journal reopen. Flush/replay evidence is not a power-loss durability guarantee. |
| Observability | Both system integration tests | Metrics are caller-reported and do not automatically instrument subsystems. |

## Validation boundary

These checks use public APIs and local process/filesystem resources. They do not claim a production server, asynchronous networking, message framing, authentication, a full TOML parser, automatic CLI task execution, hardware acceleration, or a complete production application. The TCP pipeline test sends a single fixed-width value on loopback; it is not evidence for a general wire protocol. CI remains responsible for compiling and running Rust tests/examples across configured targets.

The integration work provides executable evidence for the currently implemented public boundaries and closes roadmap item 220. It also supports items 252, 254, 255, 257 and 258. It does not claim missing framing/authentication or server-authority behavior. API/architecture freeze and review items 261–264 remain open until the contracts and implementation gaps are reviewed as a whole.

## Module dependency-cycle guard

`python3 scripts/check_module_cycles.py` scans crate-root `crate::module` references and fails CI if their directed graph contains a cycle. Unit tests exercise acyclic and cyclic graphs. This is a source-reference guardrail; it does not parse Rust syntax fully and does not prove that conceptual or runtime dependencies are acyclic. The broader architecture review remains open.
