# dinah Memory — practical guide

The `dinah::memory` module offers contiguous buffers with a fixed logical capacity. The subsystem remains experimental: its API is not considered stable. This guide describes existing primitives, not planned capabilities.

## Choosing a structure

| Need | Initial option |
|---|---|
| Homogeneous sequence with a fixed element limit | `Buffer<T>` |
| Two contiguous fields traversed separately | `SoaBuffer<A, B>` |
| Sharing mutable state across threads | Wrap explicitly in `dinah::runtime::Shared<T>` |
| Automatic growth or a strict byte limit | Not covered by these buffers |

If almost all fields of an entity are consumed at once, a `Vec<MyStruct>` or an AoS structure can be simpler. SoA is a candidate for traversing one field of many rows, not a universal optimization. Measure with a representative workload.

## `Buffer<T>`

`Buffer<T>` owns its elements and reserves capacity through `try_with_capacity`. The capacity is a logical maximum of elements, not an exact byte budget or a resident-memory guarantee. The buffer does not grow automatically.

```rust
use dinah::memory::{Buffer, BufferCreateError, BufferPushError};

fn main() -> Result<(), BufferCreateError> {
    let mut values = Buffer::try_with_capacity(3)?;
    values.try_push(10).expect("there is room");
    values.try_extend_from_slice(&[20, 30]).expect("the batch fits");
    assert_eq!(values.as_slice(), &[10, 20, 30]);

    let rejected = values.try_push(40).expect_err("the buffer is full");
    assert_eq!(rejected, BufferPushError::Full(40));

    values.clear(); // destroys the elements, keeps the reservation
    values.try_push(50).expect("the reservation is reused");
    Ok(())
}
```

- `try_with_capacity(n)` returns `BufferCreateError::AllocationFailed` if it cannot reserve the requested capacity.
- `try_push(value)` inserts one element or returns `BufferPushError::Full(value)`, keeping the rejected value.
- `try_extend_from_slice(slice)` requires `T: Clone`. If the batch does not fit, it returns `BufferExtendError { requested, remaining }` without modifying the contents due to that capacity error.
- `as_slice()` and `as_mut_slice()` lend the initialized elements. The mutable view allows changing values, not length or capacity.
- `len()`, `is_empty()`, `capacity()` and `is_full()` describe the logical state.
- `pop()` transfers the last element to the caller; `clear()` destroys all current elements and keeps the reservation.

Zero capacity is valid: the buffer is empty and full at the same time, and accepts no insertions. The type does not implement `Clone`; moving it transfers ownership. Rust's borrows prevent modifying the buffer while an incompatible view is still alive.

## `SoaBuffer<A, B>`

`SoaBuffer` stores two contiguous columns. Logical row `i` corresponds to the pair `(first[i], second[i])`; both columns keep the same length and capacity.

```rust
use dinah::memory::{SoaBuffer, SoaCreateError};

fn main() -> Result<(), SoaCreateError> {
    let mut entities = SoaBuffer::try_with_capacity(3)?;
    entities.try_push(100_u32, 1.5_f32).expect("there is room");
    entities
        .try_extend_from_slices(&[200, 300], &[2.5, 3.5])
        .expect("columns match and the batch fits");

    let (ids, positions) = entities.as_slices();
    assert_eq!(ids, &[100, 200, 300]);
    assert_eq!(positions, &[1.5, 2.5, 3.5]);
    Ok(())
}
```

- `try_with_capacity(n)` reserves both columns; if one reservation fails, no partially constructed object is returned.
- `try_push(a, b)` inserts a complete row or returns both values in `SoaPushError { first, second }`.
- `try_extend_from_slices(a, b)` requires `A: Clone` and `B: Clone`. It rejects mismatched lengths (`SoaExtendError::LengthMismatch`) or a batch that does not fit (`CapacityExceeded`) before modifying the columns.
- `as_slices()` and `as_mut_slices()` lend both columns without allowing their lengths to change.
- `len()`, `is_empty()`, `capacity()` and `is_full()` describe the logical number of rows; `pop()` extracts the last pair and `clear()` removes the rows while keeping the reservation.

The no-modification guarantees apply to normal length/capacity errors. `Clone` may run user code and panic; these operations promise no rollback against a panic during cloning.

## Observable determinism coverage (point 089)

`tests/memory_determinism.rs` repeats sequences over the public API of `Buffer` and `SoaBuffer`: insertion up to the limit, rejection without content loss, last-element extraction, `clear`, capacity reuse and column alignment when inserting in batches and extracting rows. It also repeats extension errors by capacity/length, checking that the state does not change, and the zero-capacity buffer contracts (empty and full simultaneously). The checks validate deterministic observable states for the same operation sequence; they do not imply internal synchronization or a thread-order guarantee.

## Ownership, reservation and concurrency

The buffers are owners and include no internal synchronization. Reservations use standard containers and fallible allocation operations; there is no custom allocator, automatic growth or raw pointer access. The requested capacity does not represent an exact byte limit.

On a single thread, use the buffer directly. If several threads must modify the same state, use `dinah::runtime::Shared<T>` and its `lock()` explicitly, treating the synchronization error as part of the control flow. Keep the mutex guard only during the necessary access. Composite operations, such as inserting a SoA row, must run under the same guard to preserve invariants. A mutex guarantees no fairness, absence of starvation or better performance.

`clear()` destroys values; it is not a generic reset and does not guarantee physically erasing sensitive bytes. The slices only cover initialized values and cannot outlive the owner. The implementation avoids `unsafe` and follows Rust's ownership rules.

## Benchmarks and scope

The reproducible benchmark is in `benches/memory.rs`. Historical results and their notes are kept in [`artifacts/benchmarks/`](../artifacts/benchmarks/): [Memory baseline](../artifacts/benchmarks/memory-baseline-2026-10-10.txt) and [SoA batch insertion comparison](../artifacts/benchmarks/memory-soa-batch-2026-10-11.txt). The measurements describe concrete runners and conditions; they are not universal guarantees and must not replace measurements of the real workload.

Pools, arenas, dynamic growth, strict byte limits, custom allocators, SIMD and ECS are not part of this initial API. See the [Memory design model](memory-model.md) for invariants and deferred decisions, and [`puntos.md`](../puntos.md) for the roadmap status.
