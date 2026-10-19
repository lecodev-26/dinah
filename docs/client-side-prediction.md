# Client-side prediction — research (point 127)

Status: research and first generic implementation of points 128–131. `dinah::network::prediction::PredictionBuffer` implements local prediction, bounded history, reconciliation and replay through a consumer callback. It does not transmit inputs and provides no domain simulation, authoritative server or cross-platform determinism guarantee.

## Problem and goal

In an authoritative client-server model, waiting for an input to travel to the server and its result to return adds at least the round-trip latency plus processing time to the controls. Client prediction immediately runs a local simulation of the player's own inputs so the controls feel responsive. That simulation is provisional: the server retains authority and may reject inputs or produce a different state.

Prediction is not reconciliation. Prediction estimates the result before receiving confirmation; reconciliation incorporates a later authoritative state and re-applies the still-unconfirmed inputs to recover the local present.

## Recommended model for DINAH

Adopt for the first iteration an **authoritative server + numbered inputs + bounded local prediction + reconciliation by confirmed sequence** model. It must be an explicit layer over Simulation, not an implicit responsibility of `network::Connection`.

1. The client creates a domain input with a monotonic sequence and logical tick. The input represents a validatable intent (for example, a movement direction), not a final position the client can impose on the server.
2. The client stores the input in a bounded history and immediately applies it to its local predictive state.
3. The transport sends inputs to the server. Delivery, ordering, duplication and loss must be defined in the later protocol; the existing TCP only offers a byte stream and is not equivalent to an input protocol.
4. The server validates and processes inputs according to its authoritative policy and publishes a state snapshot with the last processed sequence number for that client.
5. On receiving a snapshot, the client installs the authoritative state as the base, discards acknowledged inputs and re-simulates in order the pending inputs after the acknowledgement.
6. The corrected simulation is the game state. Any visual smoothing must be a separate presentation layer: it must not contaminate the state used for physics, collisions, rules or future inputs.

## Data and conceptual contracts of the application protocol

- **Input:** sequence, logical tick or time reference and a validated domain payload. The exact types belong to the application protocol, not the TCP wrapper.
- **Authoritative snapshot:** server tick, the minimum state needed to resume the simulation and the last processed input sequence. It must be possible to distinguish a stale, duplicated or incompatible snapshot.
- **History:** a ring buffer or bounded structure that keeps pending inputs; each entry has a stable identity. The capacity, overflow and entry-loss policy must be explicit, without growing without limit from remote data.
- **Clock:** use explicit `Simulation` ticks; do not depend on wall-clock, packet arrival order or hidden timers to produce a reproducible replay.
- **Authority and security:** the server validates every input and the definitive state. Client prediction grants no permissions and is not reliable evidence for economy, combat, inventory or competitive outcomes.
- **Errors and limits:** invalid inputs, impossible sequences, stale snapshots, tick jumps, exhausted history and incompatible versions need explicit policies. Do not hide a failure by clearing the history and pretending a correct reconciliation.
- **Integration:** Networking transports bytes; the future codec validates the wire format; a protocol layer maps messages to inputs/snapshots; Simulation executes the state transition. Avoid circular dependencies and hidden world mutation.

## Determinism: a conditional requirement, not an assumption

The replay will only reconstruct the same result if the simulated model is sufficiently reproducible for the initial state and the same input sequence. This must be investigated per model: iteration order, floating-point arithmetic, external state, randomness, concurrency and side effects. Bit-for-bit determinism across architectures must not be declared without specific tests. If the simulation is not deterministic, reconciliation can still correct the state, but resimulation may diverge and need more complete snapshots or a different design.

## Correction and visual experience

The safe default is to correct the simulation state to the authority and re-execute pending inputs. A small damped visual offset can disguise small corrections, but it must not alter the simulation truth or indefinitely hide a large divergence. For large corrections, invalid state or lost history, prefer explicit convergence over aesthetic smoothness.

Local player prediction does not solve the visualization of other entities. Remote snapshot interpolation and lag compensation are related but separate problems; they must not creep into point 128.

## Risks and limits

- Collisions with other players, server-moved objects, simultaneous events and unpredictable rules produce legitimate corrections.
- A client may run inputs locally that the server later rejects; the game must separate provisional feedback from irreversible authoritative effects.
- TCP can suffer head-of-line blocking when a segment is lost; the current transport offers no real-time guarantees and is not an input delivery policy.
- A history that is too short prevents replay; an unlimited one exposes the client to growing memory consumption. Capacity must derive from a configured time window and maximum.
- It is not correct to assume that prediction will eliminate lag, prevent cheating or guarantee exact synchronization.

## Initial implementation (points 128–131)

The API is in `src/network/prediction.rs` and is exported as `dinah::network::prediction`. `PredictionBuffer<S, I>` keeps generic state, assigns sequences from 1, applies each input as soon as it is predicted and retains pending entries until a snapshot acknowledges them. Capacity is configured when building the buffer; if it fills, `predict` returns `HistoryFull` without applying or assigning the new input.

`AuthoritativeSnapshot<S>` includes the tick, the last processed sequence and the authoritative state. `reconcile` rejects acknowledgements above the last emitted sequence, ignores stale snapshots and, if it accepts the snapshot, removes acknowledged entries and re-applies the remaining ones in original order. The consumer provides the transition function, which must avoid side effects and be reproducible for the replay to make sense.

The tests in `tests/network_prediction.rs` cover immediate application, sequences, reconciliation, replay, stale snapshots, the history limit, future ACKs and repeatability with the same transition function. This demonstrates repeatability in the test model, not universal bit-for-bit determinism.

## Work sequence for points 128–131

- **128 — Implement prediction:** initial generic implementation completed and described above. Integration with sockets, domain adapters and an application protocol remain outside the current scope.
- **129 — Implement reconciliation:** define the authoritative snapshot and confirmed sequence; apply the snapshot and re-execute only the inputs after that sequence. Initial implementation and stale/invalid-ACK tests completed.
- **130 — Implement history/state buffers:** bound capacity and validate sequences; the initial implementation rejects overflow without silently discarding inputs. The buffer does not retain data without limit.
- **131 — Implement deterministic resimulation:** repeatable replay with the same base/inputs via a shared callback and a repeatability test. The demonstrated scope does not include cross-platform determinism.

The initial tests must be deterministic and local, with delays, duplicates, loss, reordering and stale snapshots simulated through explicit inputs; they must not depend on sleeps or external services. Production loss/latency scenarios also belong to points 132–134.

## Research acceptance criteria

- Explicit separation between prediction, reconciliation, history, resimulation and visual interpolation.
- Server authority and limited client trust documented.
- Conceptual separation between sequences, snapshots and pending inputs; the generic `PredictionBuffer` API exists, but it implements no network protocol or domain adapters.
- Determinism, memory limits, errors, unpredictable events and correction policies identified.
- Incremental plan for 128–131 and deterministic tests defined.

## References consulted

- Gabriel Gambetta, [Client-Side Prediction and Server Reconciliation](https://www.gabrielgambetta.com/client-side-prediction-server-reconciliation.html): numbered inputs, authoritative state and replay of unconfirmed inputs.
- Glenn Fiedler, [What Every Programmer Needs To Know About Game Networking](https://www.gafferongames.com/post/what_every_programmer_needs_to_know_about_game_networking/): circular state/input history and rewind/replay after a correction.
- Gabriel Gambetta, [Entity Interpolation](https://www.gabrielgambetta.com/entity-interpolation.html): local entity prediction versus remote entity interpolation.
- Glenn Fiedler, [Snapshot Interpolation](https://gafferongames.com/post/snapshot_interpolation/): limits of assuming perfect determinism and the alternative of interpolating snapshots.

The references ground the conceptual model; they do not imply copying their implementations or guarantee that their parameters are suitable for DINAH.
