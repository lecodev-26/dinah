# dinah Networking — API design and v0.1 limits

Status: incremental implementation (points 120–140). The transport, message and wire format contracts are documented; a blocking TCP connection, a listening/accepting TCP endpoint and local connection-lifecycle tracking exist. The wire encoder/decoder remains unimplemented. Generic client prediction and reconciliation are implemented in `src/network/prediction.rs`; the design, limits and scope are in `docs/client-side-prediction.md`. Explicit application-level retry primitives, RTT estimation, logical liveness/backoff tracking, bounded dispatch through Runtime and tick-based delivery simulation connected to Simulation were added. They are not automatically integrated with the TCP transport or with a concrete domain model.

## Runnable example

`examples/network_basics.rs` starts an `Endpoint` on loopback with an ephemeral port, accepts a connection, exchanges `ping`/`pong` and observes EOF after the server's write shutdown. Run it with `cargo run --example network_basics`. The demonstration uses blocking TCP and a fixed-length message only to illustrate reads/writes; DINAH adds no framing, TLS, authentication, retries or sandbox. Do not expose this example as a public network service.

## Implemented primitives — points 132–139

### Application-level message retries — point 132

`network::reliability::ReliableOutbox<T>` keeps a finite number of messages with monotonic sequences, last-send tick and attempt counter. `acknowledge(through)` removes messages up to a cumulative ACK; `due_retries(now_tick, retry_interval, max_attempts)` returns the sequences the caller must resend. The queue rejects zero capacity, saturation and zero retry intervals; it does not silently discard pending messages.

This is retry bookkeeping **over an application protocol**. TCP already manages retransmission of its own stream's segments; this API does not observe IP packet loss, does not implement UDP and does not guarantee exactly-once delivery. The caller must define the ACK format, execute the sends and decide what to do when attempts are exhausted.

### Latency — point 133

`network::latency::LatencyEstimator` takes explicit RTT samples and exposes the sample count, minimum, maximum and an integer exponential smoothing (the first sample initializes; afterwards 7/8 of the previous value and 1/8 of the new sample). It reads no clock and does not measure the network by itself; the consumer measures the interval and provides the duration.

### Disconnects and backoff — point 134

`network::disconnect::ConnectionMonitor` compares a consumer-supplied logical time with the last activity instant and a configured timeout. `ReconnectBackoff` computes bounded exponential delays. Neither creates timers, detects a remote close without caller activity, opens sockets or reconnects automatically. The decision to declare a peer dead and reconnect still belongs to the application.

### Runtime — point 135

`network::runtime::NetworkWorkQueue<E>` stores one-shot work in a fixed-capacity queue and dispatches it explicitly through `runtime::Scheduler`. Dispatch is synchronous; no network thread, polling loop or background task is created. If full, `enqueue` returns the rejected work to the caller.

### Simulation and the deterministic simulator — points 136 and 138

`network::simulator::NetworkSimulator<T>` schedules payloads by delivery tick. `send(now_tick, latency_ticks, payload, drop)` takes an explicit loss decision so tests are reproducible; delivered messages are ordered by arrival tick and send order. The loss and delivery counters are observational, not metrics of a real connection.

`receive_due_for_simulation(&Simulation<S>)` uses the current logical tick to obtain the due payloads, but does not modify `Simulation`'s state: the consumer decides how to validate and apply each message. The simulated queue does not model congestion, jitter, duplication, configurable reordering or a random generator. It is an initial testing tool, not a complete network emulator.

### Tests and benchmarks — points 137 and 139

`tests/network_advanced.rs` covers limits and ACK/retry sequences, RTT estimation, logical timeout, backoff, Runtime dispatch, scripted loss, tick latency and delivery order. `benches/network.rs` measures the local cost of recording RTT samples and of enqueueing/acknowledging 64 messages. The benchmark runner runs this suite on five platforms; archived outputs exist, for example for commit [`735ee2c`](../artifacts/benchmarks/735ee2cdc221fcf3c6f3a6036d33c1033264f986/run-39171110759/). The figures are observations specific to that runner and this synthetic workload, not performance guarantees or absolute cross-platform comparisons.

## Goal and current state

Networking separates byte transport, wire format and application logic. Blocking TCP, explicit reliability/liveness helpers, a tick-delivery simulator and generic prediction/reconciliation already exist. These components do not yet form an integrated protocol: the wire encoder/decoder and framing, authentication and domain adapters remain pending.

## Responsibilities and boundaries

- **Core:** truly shared types and errors. It does not depend on Networking and contains no transport policy.
- **Runtime:** executes explicit work. Networking must not create a hidden scheduler, persistent pool or implicit background tasks.
- **Simulation:** provides state and simulation steps; Networking does not modify Simulation's state implicitly.
- **Networking:** defines endpoints, connections, messages, transport and serialization contracts, plus network errors.
- **Application:** decides authentication/authorization, the game or business protocol, trust limits and how to treat valid but unauthorized messages.

The first implementation must use safe Rust, with no `unsafe` or external dependencies until a concrete requirement and a license/security review justify an exception.

## Conceptual layered API draft

The following block keeps the initial design as an architectural reference; it is not a reliable list of currently available types and must not be used as a compilable API example. For the real state, see "Implemented primitives" and "Current state and scope". The only implementation claims are those described in those sections and in the source code's signatures.

```rust
pub trait Transport {
    type Address;
    type Connection;
    type Error;

    fn connect(&mut self, address: Self::Address)
        -> Result<Self::Connection, Self::Error>;
    fn send(&mut self, connection: &mut Self::Connection,
            frame: &[u8]) -> Result<(), Self::Error>;
    fn receive(&mut self, connection: &mut Self::Connection,
               buffer: &mut [u8]) -> Result<usize, Self::Error>;
}
```

This signature is a discussion point, not a final API promise. Point 121 already fixes the first transport as a blocking byte stream, with partial reads and explicit EOF; the definitive signatures must reflect that contract and must not treat one write call as if it delimited a complete message. The error types and the concrete provider will be decided when implementing connections.

Planned layers:

1. **Logical message:** message type/version and payload, without coupling it to TCP, UDP or an in-memory Rust structure.
2. **Serialization:** encodes/decodes bytes with explicit limits, length validation and typed errors. Never trust sizes provided by the peer.
3. **Transport:** delivers bytes and declares stream/datagram semantics, blocking, ordering, errors and close.
4. **Connection and endpoint:** local/remote identity and locally observed connection state, without confusing address, authenticated session and authorization.
5. **Integration:** explicit adapters toward Runtime and Simulation; no hidden mutation or implicit background callback.

## Transport decision — point 121

For the first version, a **blocking, explicit byte-stream transport** is defined, with read and write operations that may complete a partial amount of bytes. The contract is independent of the concrete provider: the first socket implementation belongs to the connections point and is not part of this design point. No UDP or common stream/datagram abstraction is added yet; those models have distinct semantics and must not be hidden behind the same API.

Chosen contract:

- **Complete write:** the high-level `write_all(bytes)` operation only returns success when all requested bytes have been accepted. An internal implementation may make several partial writes. If an error occurs after writing a prefix, it returns the error and does not secretly retry from scratch: the caller must treat the stream state as potentially desynchronized and usually close the connection.
- **Read:** `read(buffer)` returns the number of bytes actually read, which may be smaller than the buffer size. `0` means EOF/ordered close by the peer, not "no data yet". Partial reads do not delimit messages; framing belongs to a later layer.
- **Blocking:** operations wait for progress or a system error. The first API does not expose `WouldBlock` as a normal flow and promises no non-blocking operations; an async evolution will require another explicit contract.
- **Timeouts:** no clock or global deadline is hidden. If the provider allows configuring a timeout, it is set explicitly in its configuration and the timeout is returned as a typed error. No portable maximum latency is promised.
- **Close:** EOF indicates a remote read-side close. Write/read errors are preserved as errors; they are not converted into an empty buffer. Local close must be an explicit operation of the connection provider, specified when implementing connections.
- **Ownership and concurrency:** bytes are borrowed only during the call and are not retained. The connection object is mutable and not shared concurrently without external synchronization. Byte order is preserved by the stream's semantics, but one read call does not necessarily correspond to one write.
- **Limits:** the transport moves bytes and allocates no memory based on remote lengths. The framing/serialization layer defines and validates maxima before reserving payloads.
- **Partial failures:** after a partial write failure, the protocol state is considered uncertain; the complete frame is not automatically retried. Recovery or reconnection is outside the base contract.

This point closes a design decision, not a network implementation. Contract tests may use a fake provider; local socket tests belong to point 137 and must not depend on exact timings or external services.

## Logical message contract — point 122

A logical message is a protocol unit independent of the concrete transport and byte layout. Reading the stream does not create messages: the framing that will allow reconstructing them is specified in point 123. This point defines the message's semantics and identity, not its binary encoding.

### Conceptual envelope

Every message must have these semantic elements:

- **Protocol version:** identifies the version of the common Networking contract that interprets the envelope. It is not the version of a message type.
- **Type identifier:** a stable numeric identifier assigned to a message class within the protocol. The zero value is reserved as invalid; a published identifier is not reused for another meaning.
- **Message version:** the semantic schema version of the identified type, separate from the global protocol version.
- **Opaque payload:** type-specific content. The message layer interprets no in-memory Rust structs and does not assume the payload has a portable representation without explicit serialization.

As design notation, not an already-implemented API:

```rust
struct MessageEnvelope<'a> {
    protocol_version: ProtocolVersion,
    message_type: MessageType,
    message_version: MessageVersion,
    payload: &'a [u8],
}
```

These names illustrate the conceptual contract; they do not yet exist as public Rust types. The exact field representation and order, endianness, integer encoding and framing are fixed in point 123.

### Validation and evolution rules

- The receiver validates the protocol version, type identifier, message version and payload size before delivering the message to application logic.
- An unknown type or unsupported message version produces a typed error by default; ignoring it will only be allowed through an explicit protocol compatibility policy, never by implicit silent discard.
- An incompatible protocol version is rejected before dispatching the payload. Forward/backward compatibility rules must be declared per version; they are not inferred from the fact that the payload can be decoded.
- Published identifiers are stable and are not reassigned. An incompatible modification of meaning requires a new identifier or a new message version, according to the documented compatibility rules.
- The maximum payload and complete message size must be finite and checked before reserving memory based on remote data. The concrete value and length representation are fixed in point 123; this point does not yet invent a numeric limit.
- Message errors distinguish, at minimum, unsupported protocol version, unknown type, unsupported message version and exceeded limit. Framing and decoding errors are detailed in serialization.
- Ordering is only what the selected transport guarantees. This envelope adds no sequences, acknowledgements, exactly-once delivery or deduplication.
- A valid message implies no authentication, authorization, trust in the sender or encryption. The application decides the security policy and validates the domain data.
- Connection identity, session state, lifecycle control messages, ACKs/retries and reconnection are outside the base envelope and are decided in the connections/state points or in an explicit application protocol.

### Acceptance criteria for point 122

- The envelope's semantic fields are defined and protocol version and message version are separated.
- Rules are fixed for stable identifiers, unknown types/versions, compatibility and limits without prematurely deciding the binary encoding.
- Message errors are distinguished from framing/serialization errors and no delivery guarantees the transport does not provide are promised.
- It is expressly declared that the Rust example is conceptual and that no message types are implemented yet.

## TCP endpoint — point 125

`dinah::network::Endpoint` wraps a `std::net::TcpListener` and allows creating a listening socket, querying the local address, explicitly configuring non-blocking mode and accepting incoming connections as `Connection`. It starts no background tasks and keeps no registry of accepted connections.

- `Endpoint::bind(SocketAddr)` binds the requested address; port `0` lets the operating system assign an available port.
- `local_addr()` reports the effective local address, including the assigned port.
- `accept()` returns an owning `Connection` for the accepted `TcpStream` and the peer address. By default it blocks until a connection or operating-system error.
- `set_nonblocking(true)` makes non-blocking mode explicit; `accept()` may then return `WouldBlock` if there are no pending connections. No polling or scheduler is added.
- The endpoint adds no framing, serialization, authentication, encryption, reconnection or state management.

### Acceptance criteria for point 125

- Public API for bind, local address, accept and explicit non-blocking mode.
- Accepted connections reuse the already-implemented `Connection` wrapper.
- Loopback tests cover bind/accept/byte exchange and the explicit `WouldBlock` behavior.
- No background tasks are created and no claim is made that connection-state management exists.

## TCP connections — point 124

The first concrete implementation is `dinah::network::Connection`, an owning wrapper of `std::net::TcpStream`. It only provides byte-stream operations; it does not turn writes into messages and does not yet incorporate point 123's v1 framing.

- `Connection::connect(SocketAddr)` connects using the operating system's default timeout behavior.
- `Connection::connect_timeout(SocketAddr, Duration)` allows requesting an explicit connection timeout and rejects a zero duration as invalid input.
- `Connection::from_tcp_stream(TcpStream)` takes ownership of an already-established stream, such as one accepted by a listener.
- `read(&mut [u8])` preserves partial-read and EOF semantics; errors, including configured timeouts, are returned as `ConnectionError::Io`.
- `write_all(&[u8])` only returns success if all bytes were written. On a partial error, the stream state may be uncertain; there is no automatic retry of the complete frame.
- `set_read_timeout` and `set_write_timeout` explicitly configure operation timeouts. `None` disables the timeout, subject to platform behavior.
- `local_addr`, `peer_addr` and `shutdown` expose the socket addresses and explicit close. There is no automatic reconnection, background tasks, authentication, encryption or internal synchronization.

The methods require mutable access to read/write. The wrapper implements neither `Clone` nor implicit socket duplication. The tests use loopback and a local listener, with no external services or exact-latency assertions. This initial TCP API implies no UDP support or API stability.

### Acceptance criteria for point 124

- Public owning wrapper of `TcpStream` with normal connection and explicit-timeout connection.
- Byte operations, EOF, errors, addresses and close explicit and documented.
- Loopback tests for byte round-trips, EOF, a preexisting stream and zero-timeout rejection.
- Without claiming framing, serialization, authentication, reconnection or endpoints as already-implemented capabilities.

## Mandatory contracts

- **Ownership:** buffers borrowed during the call, unless a future API explicitly documents an ownership transfer. Do not retain references to caller buffers.
- **Framing:** the stream needs its own delimitation (prefixed length or other defined framing); read boundaries are not message boundaries. Datagrams keep the datagram's limits, subject to the configured maximum.
- **Limits:** documented configurable/constant maximum frame and message length; reject overflow and truncated input before allocating memory based on remote data.
- **Errors:** distinguish invalid configuration, transport failure, close/EOF, invalid frame, invalid serialization and exceeded limit. Do not convert errors into empty payloads or retry silently.
- **Time:** timeouts and deadlines must be explicit. The first API must not query a hidden global clock or promise maximum latency.
- **Concurrency:** do not share a mutable connection across threads without explicit synchronization. Ordering is only guaranteed where the selected transport promises it.
- **Security:** framing is not authentication or encryption. Do not trust the peer; do not expose keys/tokens in logs; do not enable deserialization of arbitrary types.
- **Close:** define when a connection is open/closed and how EOF, local close and remote close propagate before promising automatic recovery.
- **Determinism:** protocol tests use buffers and a fake transport; socket tests must not depend on exact timings or external availability.
- **Portability:** the high-level API must not leak operating-system-specific socket types if it can express the contract portably.

## Deferred decisions and implementation order

- **121 Transport (decision taken):** blocking byte stream; partial reads allowed, EOF defined, complete write with partial-progress errors, explicit provider timeout and no automatic retry. Stream is not mixed with datagrams and no claim is made that sockets are implemented.
- **122 Messages (decision taken):** conceptual envelope with protocol version, stable type identifier, message version and opaque payload; binary encoding and framing remain for 123.
- **123 Serialization (decision taken):** framing by 32-bit big-endian prefixed length; 6-byte header with protocol version, type and message version as 16-bit big-endian integers; maximum frame of 1 MiB including the header; explicit rejections of invalid length, truncation and invalid versions/types. Minimal custom format, with no external dependency or Rust-struct serialization.
- **124 Connections (initially implemented):** `dinah::network::Connection` wraps `TcpStream` with blocking read/write, explicit timeout, addresses and shutdown; no integrated framing or automatic reconnection. **125 Endpoints (initially implemented):** `dinah::network::Endpoint` wraps `TcpListener` with bind, local address, accept and explicit non-blocking mode. **126 State management (initially implemented):** `ConnectionState` records the locally observable state (`Open`, `ReadClosed`, `WriteClosed`, `Closed`); it does not immediately detect a remote close and manages no reconnection. **127 Prediction (research completed):** model and limits in `docs/client-side-prediction.md`. **128–131 Prediction/reconciliation/history/resimulation (initial generic implementation):** `network::prediction::PredictionBuffer` keeps a bounded history, applies inputs locally and reconciles authoritative snapshots by replaying pending inputs. The state transition is provided by the consumer; there is no domain adapter or cross-platform determinism guarantee.
- **132–134 Loss/latency/disconnect (initial primitives implemented):** application-level retry outbox, RTT estimator and logical monitor/backoff. No automatic reconnection or TCP retransmission control.
- **135–136 Runtime/Simulation integration (explicit adapters implemented):** bounded work queue with synchronous dispatch and tick-based simulated delivery; the consumer applies the messages.
- **137–139 Tests/latency/benchmarks (initial suite implemented):** deterministic tests of policies and simulator, estimator/outbox benchmark integrated in the runner; cross-platform measurements depend on CI.
- **140 Documentation:** document only implemented behavior and measured limits.

## Acceptance criteria for point 120

Point 121 is defined when the transport contract fixes stream semantics, partial read/EOF, partial write, blocking, timeout, close, ownership, limits and recovery after error, without presenting it as an implementation.

- Separated responsibilities of Core, Runtime, Simulation and Networking.
- Logical layers of message, serialization, transport, connection and integration identified.
- Initial contracts of ownership, framing, limits, errors, time, concurrency, close, security and determinism defined.
- Unresolved decisions labeled as proposals, not existing APIs.
- No transport, socket, serializer or prediction feature is considered implemented by this document.

## Acceptance criteria for point 121

Point 121 is defined when the transport contract fixes stream semantics, partial read/EOF, partial write, blocking, timeout, close, ownership, limits and recovery after error.

## Serialization and wire format — point 123

This contract fixes a small, explicit wire encoding for the envelope defined in point 122. It remains a design decision: no encoder/decoder or public framing API is implemented.

### v1 binary format

All integers are encoded in **big-endian**. No in-memory Rust structs, padding, implicit discriminants, `usize` or platform-dependent representations are transmitted.

Every frame in the byte stream has this shape:

| Field | Size | Meaning |
|---|---:|---|
| `frame_length` | 4 bytes | Big-endian length of everything that follows, not including this prefix |
| `protocol_version` | 2 bytes | Protocol version; `1` is the initial version defined by this format |
| `message_type` | 2 bytes | Stable type identifier; `0` is invalid/reserved |
| `message_version` | 2 bytes | Message schema version; `0` is invalid/reserved |
| `payload` | 0 or more bytes | Opaque message bytes, defined by the type and version |

`frame_length` counts the six header bytes and the payload, but not its own four bytes. The valid minimum is 6 bytes. The maximum frame is **1 MiB (1,048,576 bytes)**, including the header but excluding the length prefix; therefore the maximum payload is 1,048,570 bytes. The maximum total size on the stream is 1,048,580 bytes counting the prefix. These limits are constants of the v1 format, not trustworthy data sent by the peer.

### Decoding and framing

- The decoder reads exactly the four prefix bytes, accepting partial stream reads through explicit accumulation.
- It validates `frame_length` against the minimum and maximum before reserving or accumulating the body. An out-of-range length is rejected; it is not truncated, saturated or used to attempt an allocation.
- It then reads exactly the declared body. EOF before completing prefix or body is reported as a truncated frame; no partial message is delivered.
- The decoder validates the protocol version, type and message version before dispatching the payload. Unsupported protocol version, zero/unknown type and zero/unsupported message version are rejected through differentiated errors.
- A valid frame consumes exactly the declared prefix and body. The following bytes belong to the next frame; a length must not include bytes of a later frame.
- The encoder computes the length with overflow checking, rejects messages exceeding the maximum and emits the prefix and header in the defined order. It cannot depend on native endianness.
- The payload stays opaque at the wire layer. Decoding of each message's own fields will use specific rules and limits; deserialization of arbitrary types is not enabled and Rust memory is not interpreted directly.
- No checksum, compression, encryption, authentication, ACK, sequences or recovery after loss is defined. Those functions require separate contracts and must not be inferred from this format.

### Minimal serialization errors

The future API must distinguish at least: frame length too small, frame length exceeded, overflow while computing length, truncated prefix/body, unsupported protocol version, invalid or unknown type, and invalid or unsupported message version. Underlying transport failures are preserved as transport failures, not disguised as payload errors.

### Compatibility and evolution

The described layout is wire format v1. Changing the size, order, endianness or semantics of the header fields is an incompatible change and requires a new protocol version or an explicit migration decision. New types use non-reused identifiers; incompatible changes in a payload schema increment its message version or receive a new identifier according to the protocol's policy. Guessing versions or ignoring unknown header fields is not allowed.

### Acceptance criteria for point 123

- Binary format, byte order, framing and length meaning defined without ambiguity.
- Minimum/maximum limits validated before allocations based on remote data.
- Partial reads, EOF/truncation, overflow and version/type errors have explicit behavior.
- Depending on native Rust layout or adding dependencies without justified need is prohibited.
- It is declared that this point only decides the contract: the encoder/decoder implementation and code tests will be done in later implementation/testing points.

## Current state and scope — point 140

Points 120–139 have decisions and incremental primitives documented above; point 140 closes this Networking documentation without claiming a complete stack exists. Today there are blocking TCP wrappers (`Connection`, `Endpoint`), local connection state, an explicit work queue toward Runtime, RTT estimation, a liveness/backoff monitor, an application retry outbox, generic prediction/reconciliation and a deterministic tick simulator.

Still pending are the wire format's framing/encoder/decoder, authentication, encryption, a complete protocol policy, UDP, async operations, automatic connection management and end-to-end delivery guarantees. The TCP API moves bytes and does not delimit messages; the reliability and simulation helpers are not automatically connected to the socket. The simulator uses loss decisions supplied by the consumer and models no congestion or implicit randomness.

The Networking code avoids external dependencies and `unsafe`. Timeouts, clocks, retries and dispatch are activated explicitly from the caller. This separation allows testing each contract separately and avoids presenting future functions as already-available capabilities.
