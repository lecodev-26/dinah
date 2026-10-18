# DINAH integration matrix (v0.1 work in progress)

This records executable integration evidence without implying automatic wiring between all subsystems.

| Boundary | Executable evidence | Guarantee / limitation |
|---|---|---|
| Runtime → Network | `tests/network_advanced.rs`, `tests/system_integration.rs` | Caller-created one-shot work dispatches synchronously through `Scheduler`; enqueue alone does not execute it and no background thread/socket is created. |
| Runtime → Native | `tests/system_integration.rs` | Host OS, architecture, process ID and monotonic elapsed time can be consumed inside scheduled work; this is host metadata, not direct hardware access. |
| CLI → manifest | `tests/system_integration.rs` invokes the actual `dinah` binary | `init` creates a manifest, `inspect --format json` reads it, and a second `init` must not overwrite it. `run` still does not execute declared tasks. |
| Storage and application data | `tests/system_pipeline.rs` | Explicit FIFO storage work writes a simulation snapshot to `MemoryStore`; this is not a durability guarantee for disk storage. |
| Observability | Both system integration tests | Metrics are caller-reported and do not automatically instrument subsystems. |

## Validation boundary

These checks use public APIs and local process/filesystem resources. They do not claim a production server, asynchronous networking, message framing, authentication, a full TOML parser, automatic CLI task execution, hardware acceleration, or a complete production application. CI remains responsible for compiling and running Rust tests/examples across configured targets.

The integration work provides evidence for roadmap items 220, 252, 254, 255, 257 and 258. API/architecture freeze and review items 261–264 remain open until the contracts and implementation gaps are reviewed as a whole.
