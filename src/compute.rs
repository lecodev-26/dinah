//! Sequential, explicit data-processing primitives.
//!
//! Operations borrow their input slices and never create worker threads. The
//! [`map`] function is the exception to allocation-free processing: it returns
//! a newly allocated `Vec` whose length matches the input slice.

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
    use super::{for_each, for_each_batch, map, try_reduce, BatchError};

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
