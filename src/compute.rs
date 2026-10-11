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
    use super::{for_each, map, try_reduce};

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
