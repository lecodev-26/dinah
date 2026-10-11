//! Sequential, explicit data-processing primitives.
//!
//! Operations borrow their input slices. Sequential functions do not create
//! worker threads; [`for_each_parallel`] opts into bounded Runtime workers. The
//! [`map`] function returns a newly allocated `Vec` whose length matches the input slice.

/// Applies a mutable operation to every element, from left to right.
///
/// This function does not allocate. Each element is passed to `operation`
/// exactly once during normal completion. If the closure panics, the panic is
/// propagated and later elements are not visited; earlier mutations are not
/// rolled back.
///
/// # Examples
///
/// ```
/// use dinah::compute::for_each;
///
/// let mut values = [1, 2, 3];
/// for_each(&mut values, |value| *value *= 2);
/// assert_eq!(values, [2, 4, 6]);
/// ```
pub fn for_each<T, F>(items: &mut [T], mut operation: F)
where
    F: FnMut(&mut T),
{
    for item in items {
        operation(item);
    }
}

/// Error returned when a batch operation receives an invalid batch size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchError {
    /// Batch size must be greater than zero.
    ZeroSize,
}

impl core::fmt::Display for BatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroSize => f.write_str("batch size must be greater than zero"),
        }
    }
}

impl std::error::Error for BatchError {}

/// Applies an operation to successive mutable batches without allocating.
///
/// `batch_size` must be greater than zero. Validation happens before the first
/// callback, so a zero size never partially mutates the input. Each callback
/// receives a mutable slice into the original input; no temporary batch buffer
/// is created. The final batch may be shorter than `batch_size`. Batches and
/// elements within each batch are visited in input order. An empty input with a
/// valid batch size invokes no callbacks. Panics propagate and prior mutations
/// are not rolled back.
///
/// # Errors
///
/// Returns [`BatchError::ZeroSize`] when `batch_size` is zero.
///
/// # Examples
///
/// ```
/// use dinah::compute::for_each_batch;
///
/// let mut values = [1, 2, 3, 4, 5];
/// for_each_batch(&mut values, 2, |batch| {
///     for value in batch {
///         *value *= 10;
///     }
/// })?;
/// assert_eq!(values, [10, 20, 30, 40, 50]);
/// # Ok::<(), dinah::compute::BatchError>(())
/// ```
pub fn for_each_batch<T, F>(
    items: &mut [T],
    batch_size: usize,
    mut operation: F,
) -> Result<(), BatchError>
where
    F: FnMut(&mut [T]),
{
    if batch_size == 0 {
        return Err(BatchError::ZeroSize);
    }

    for batch in items.chunks_mut(batch_size) {
        operation(batch);
    }
    Ok(())
}

/// Error returned when parallel iteration is requested without any workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelError {
    /// At least one worker must be requested.
    ZeroWorkers,
}

impl core::fmt::Display for ParallelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroWorkers => f.write_str("parallel iteration requires at least one worker"),
        }
    }
}

impl std::error::Error for ParallelError {}

/// Applies an operation to every element using explicitly bounded Runtime workers.
///
/// The slice is split into disjoint mutable chunks before dispatch. Each element
/// is passed to `operation` exactly once during normal completion; completion
/// order is unspecified. The Runtime scheduler creates scoped workers for this
/// call and joins the active batch before propagating a worker panic. Mutations
/// completed before a panic are not rolled back. The operation is shared across
/// workers, so it must be `Sync`; elements must be `Send` to cross worker
/// boundaries. This function does not promise a speedup for small workloads.
///
/// `worker_count` must be positive. The invalid value is rejected before any
/// element is mutated. Empty slices with a positive worker count do no work and
/// spawn no workers.
///
/// # Errors
///
/// Returns [`ParallelError::ZeroWorkers`] when `worker_count` is zero.
///
/// # Examples
///
/// ```
/// use dinah::compute::for_each_parallel;
///
/// let mut values = [1, 2, 3, 4];
/// for_each_parallel(&mut values, 2, |value| *value *= 2)?;
/// assert_eq!(values, [2, 4, 6, 8]);
/// # Ok::<(), dinah::compute::ParallelError>(())
/// ```
pub fn for_each_parallel<T, F>(
    items: &mut [T],
    worker_count: usize,
    operation: F,
) -> Result<(), ParallelError>
where
    T: Send,
    F: Fn(&mut T) + Sync,
{
    use core::num::NonZeroUsize;
    use std::convert::Infallible;

    let worker_count = NonZeroUsize::new(worker_count).ok_or(ParallelError::ZeroWorkers)?;
    if items.is_empty() {
        return Ok(());
    }

    let active_workers = worker_count.get().min(items.len());
    let chunk_size =
        items.len() / active_workers + usize::from(!items.len().is_multiple_of(active_workers));
    let chunk_count =
        items.len() / chunk_size + usize::from(!items.len().is_multiple_of(chunk_size));
    let capacity =
        NonZeroUsize::new(chunk_count).expect("non-empty input creates at least one chunk");
    let mut queue = crate::runtime::TaskQueue::new(capacity);

    for chunk in items.chunks_mut(chunk_size) {
        let operation = &operation;
        queue
            .try_push(move || {
                for item in chunk {
                    operation(item);
                }
                Ok::<(), Infallible>(())
            })
            .unwrap_or_else(|_| unreachable!("queue capacity matches the number of chunks"));
    }

    let results = crate::runtime::Scheduler::new().run_parallel(&mut queue, worker_count);
    for result in results {
        match result {
            Ok(()) => {}
            Err(never) => match never {},
        }
    }
    Ok(())
}

/// Error returned by parallel reduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParallelReduceError<E> {
    /// At least one worker must be requested.
    ZeroWorkers,
    /// The reduction operation failed in an input partition.
    Operation(E),
}

impl<E: core::fmt::Display> core::fmt::Display for ParallelReduceError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroWorkers => f.write_str("parallel reduction requires at least one worker"),
            Self::Operation(error) => write!(f, "parallel reduction operation failed: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ParallelReduceError<E> {}

/// Reduces a slice in parallel, then combines partial accumulators in input order.
///
/// `identity` is cloned once per partition and is expected to be a valid identity
/// for both `operation` and `combine`. Each partition is reduced from that value;
/// partial accumulators are then combined from left to right in partition order.
/// For results independent of the partitioning, the operation/combine algebra
/// must be associative and the identity must be valid. Floating-point operations
/// may differ from sequential reduction because their grouping changes.
///
/// All partition tasks finish before a result is returned. If multiple partitions
/// fail, the error from the earliest partition in input order is returned. The
/// combine closure runs only if every partition succeeds. `worker_count` must be
/// positive; zero is rejected before any operation or combination is invoked.
/// Empty input returns `identity` without invoking either closure or spawning
/// workers. Panics follow the Runtime scheduler contract and are propagated.
///
/// # Errors
///
/// Returns [`ParallelReduceError::ZeroWorkers`] for zero workers or
/// [`ParallelReduceError::Operation`] for the first operation error in input order.
///
/// # Examples
///
/// ```
/// use dinah::compute::try_reduce_parallel;
///
/// let values = [1_i64, 2, 3, 4, 5, 6];
/// let sum = try_reduce_parallel(
///     &values,
///     3,
///     0_i64,
///     |sum, value| Ok::<_, &'static str>(sum + value),
///     |left, right| left + right,
/// )?;
/// assert_eq!(sum, 21);
/// # Ok::<(), dinah::compute::ParallelReduceError<&'static str>>(())
/// ```
pub fn try_reduce_parallel<T, A, E, F, C>(
    items: &[T],
    worker_count: usize,
    identity: A,
    operation: F,
    combine: C,
) -> Result<A, ParallelReduceError<E>>
where
    T: Sync,
    A: Clone + Send,
    E: Send,
    F: Fn(A, &T) -> Result<A, E> + Sync,
    C: Fn(A, A) -> A + Sync,
{
    use core::num::NonZeroUsize;

    let worker_count = NonZeroUsize::new(worker_count).ok_or(ParallelReduceError::ZeroWorkers)?;
    if items.is_empty() {
        return Ok(identity);
    }

    let active_workers = worker_count.get().min(items.len());
    let chunk_size =
        items.len() / active_workers + usize::from(!items.len().is_multiple_of(active_workers));
    let chunk_count =
        items.len() / chunk_size + usize::from(!items.len().is_multiple_of(chunk_size));
    let capacity =
        NonZeroUsize::new(chunk_count).expect("non-empty input creates at least one chunk");
    let mut queue = crate::runtime::TaskQueue::new(capacity);

    for chunk in items.chunks(chunk_size) {
        let operation = &operation;
        let initial = identity.clone();
        queue
            .try_push(move || {
                let mut accumulator = initial;
                for item in chunk {
                    accumulator = operation(accumulator, item)?;
                }
                Ok::<A, E>(accumulator)
            })
            .unwrap_or_else(|_| unreachable!("queue capacity matches the number of chunks"));
    }

    let partials = crate::runtime::Scheduler::new().run_parallel(&mut queue, worker_count);
    let mut accumulator: Option<A> = None;
    for partial in partials {
        let partial = partial.map_err(ParallelReduceError::Operation)?;
        accumulator = Some(match accumulator {
            None => partial,
            Some(previous) => combine(previous, partial),
        });
    }
    Ok(accumulator.expect("non-empty input creates at least one partition"))
}

/// Maps a borrowed slice into a newly allocated vector, preserving input order.
///
/// The returned vector contains one output per input element. This operation
/// allocates storage for the result; it does not allocate an intermediate
/// collection. Allocation failure follows the standard library's allocation
/// behavior. If the closure panics, the panic is propagated.
///
/// # Examples
///
/// ```
/// use dinah::compute::map;
///
/// let values = [1, 2, 3];
/// let squares = map(&values, |value| value * value);
/// assert_eq!(squares, [1, 4, 9]);
/// ```
pub fn map<T, U, F>(items: &[T], mut operation: F) -> Vec<U>
where
    F: FnMut(&T) -> U,
{
    let mut output = Vec::with_capacity(items.len());
    for item in items {
        output.push(operation(item));
    }
    output
}

/// Reduces a borrowed slice from left to right, propagating the first error.
///
/// The `initial` value is the accumulator before the first element. The
/// operation is called once per visited element, in slice order, and may
/// return a caller-defined error. On the first `Err`, reduction stops and that
/// error is returned unchanged. This function does not allocate. An empty
/// slice returns `Ok(initial)` without invoking the closure.
///
/// # Examples
///
/// ```
/// use dinah::compute::try_reduce;
///
/// let values = [1, 2, 3, 4];
/// let sum = try_reduce(&values, 0, |sum, value| Ok::<_, ()>(sum + value))?;
/// assert_eq!(sum, 10);
/// # Ok::<(), ()>(())
/// ```
pub fn try_reduce<T, A, E, F>(items: &[T], initial: A, mut operation: F) -> Result<A, E>
where
    F: FnMut(A, &T) -> Result<A, E>,
{
    let mut accumulator = initial;
    for item in items {
        accumulator = operation(accumulator, item)?;
    }
    Ok(accumulator)
}

#[cfg(test)]
mod tests {
    use super::{
        for_each, for_each_batch, for_each_parallel, map, try_reduce, try_reduce_parallel,
        BatchError, ParallelError, ParallelReduceError,
    };

    #[test]
    fn for_each_visits_every_element_once_in_order() {
        let mut values = [1, 2, 3, 4];
        let mut visited = Vec::new();
        for_each(&mut values, |value| {
            visited.push(*value);
            *value += 10;
        });
        assert_eq!(visited, [1, 2, 3, 4]);
        assert_eq!(values, [11, 12, 13, 14]);
    }

    #[test]
    fn for_each_accepts_an_empty_slice() {
        let mut values: [u8; 0] = [];
        for_each(&mut values, |value| *value += 1);
        assert_eq!(values, []);
    }

    #[test]
    fn for_each_batch_visits_full_and_partial_batches_in_order() {
        let mut values = [1, 2, 3, 4, 5];
        let mut batch_lengths = Vec::new();
        for_each_batch(&mut values, 2, |batch| {
            batch_lengths.push(batch.len());
            for value in batch {
                *value *= 10;
            }
        })
        .unwrap();
        assert_eq!(batch_lengths, [2, 2, 1]);
        assert_eq!(values, [10, 20, 30, 40, 50]);
    }

    #[test]
    fn for_each_batch_rejects_zero_before_mutation() {
        let mut values = [1, 2, 3];
        let mut called = false;
        let result = for_each_batch(&mut values, 0, |_| called = true);
        assert_eq!(result, Err(BatchError::ZeroSize));
        assert!(!called);
        assert_eq!(values, [1, 2, 3]);
    }

    #[test]
    fn for_each_batch_accepts_empty_input_with_valid_size() {
        let mut values: [u8; 0] = [];
        let mut called = false;
        for_each_batch(&mut values, 4, |_| called = true).unwrap();
        assert!(!called);
    }

    #[test]
    fn for_each_batch_accepts_batch_larger_than_input() {
        let mut values = [1, 2, 3];
        let mut lengths = Vec::new();
        for_each_batch(&mut values, 10, |batch| lengths.push(batch.len())).unwrap();
        assert_eq!(lengths, [3]);
    }

    #[test]
    fn for_each_parallel_visits_each_element_once() {
        let mut values = [0_u32; 257];
        for_each_parallel(&mut values, 4, |value| *value += 1).unwrap();
        assert!(values.iter().all(|value| *value == 1));
    }

    #[test]
    fn for_each_parallel_rejects_zero_workers_before_mutation() {
        let mut values = [1, 2, 3];
        let result = for_each_parallel(&mut values, 0, |value| *value += 10);
        assert_eq!(result, Err(ParallelError::ZeroWorkers));
        assert_eq!(values, [1, 2, 3]);
    }

    #[test]
    fn for_each_parallel_accepts_empty_input() {
        let mut values: [u8; 0] = [];
        for_each_parallel(&mut values, 4, |value| *value += 1).unwrap();
        assert!(values.is_empty());
    }

    #[test]
    fn for_each_parallel_caps_workers_to_input_length() {
        let mut values = [1, 2, 3];
        for_each_parallel(&mut values, 64, |value| *value *= 2).unwrap();
        assert_eq!(values, [2, 4, 6]);
    }

    #[test]
    fn try_reduce_parallel_combines_partition_results_in_input_order() {
        let values = [1_i64, 2, 3, 4, 5, 6, 7];
        let result = try_reduce_parallel(
            &values,
            3,
            0_i64,
            |sum, value| Ok::<_, ()>(sum + value),
            |left, right| left + right,
        );
        assert_eq!(result, Ok(28));
    }

    #[test]
    fn try_reduce_parallel_returns_identity_for_empty_input() {
        let values: [u8; 0] = [];
        let result = try_reduce_parallel(
            &values,
            2,
            42_u32,
            |sum, value| Ok::<_, ()>(sum + u32::from(*value)),
            |left, right| left + right,
        );
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn try_reduce_parallel_rejects_zero_workers_before_calling_closures() {
        let values = [1, 2, 3];
        let result = try_reduce_parallel(
            &values,
            0,
            0,
            |sum, value| Ok::<_, ()>(sum + value),
            |left, right| left + right,
        );
        assert_eq!(result, Err(ParallelReduceError::ZeroWorkers));
    }

    #[test]
    fn try_reduce_parallel_returns_earliest_partition_error() {
        let values = [1, 2, 3, 4, 5, 6];
        let result = try_reduce_parallel(
            &values,
            3,
            0,
            |sum, value| {
                if *value == 2 || *value == 5 {
                    Err(*value)
                } else {
                    Ok(sum + value)
                }
            },
            |left, right| left + right,
        );
        assert_eq!(result, Err(ParallelReduceError::Operation(2)));
    }

    #[test]
    fn map_preserves_order_and_maps_empty_slices() {
        assert_eq!(map(&[3, 1, 4], |value| value * 2), [6, 2, 8]);
        let empty: [u8; 0] = [];
        let output: Vec<u8> = map(&empty, |value| *value);
        assert!(output.is_empty());
    }

    #[test]
    fn try_reduce_accumulates_from_left_to_right() {
        let values = [1, 2, 3, 4];
        let mut visited = Vec::new();
        let result = try_reduce(&values, 0, |sum, value| {
            visited.push(*value);
            Ok::<_, ()>(sum + value)
        });
        assert_eq!(result, Ok(10));
        assert_eq!(visited, [1, 2, 3, 4]);
    }

    #[test]
    fn try_reduce_empty_slice_returns_initial_value() {
        let empty: [u8; 0] = [];
        assert_eq!(
            try_reduce(&empty, 42, |sum, value| Ok::<_, ()>(sum + value)),
            Ok(42)
        );
    }

    #[test]
    fn try_reduce_propagates_first_error_and_stops() {
        let values = [1, 2, 3, 4];
        let mut visited = Vec::new();
        let result = try_reduce(&values, 0, |sum, value| {
            visited.push(*value);
            if *value == 3 {
                Err("stop")
            } else {
                Ok(sum + value)
            }
        });
        assert_eq!(result, Err("stop"));
        assert_eq!(visited, [1, 2, 3]);
    }
}
