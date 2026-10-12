//! Determinism contracts for the public Compute API.
use dinah::compute::{for_each_parallel, try_reduce_parallel};

#[test]
fn associative_integer_reduction_is_repeatable_across_worker_counts() {
    let values: Vec<u64> = (0..257).map(|value| value as u64).collect();
    let expected: u64 = values.iter().sum();

    for worker_count in [1, 2, 3, 7, 64] {
        for _ in 0..32 {
            let result = try_reduce_parallel(
                &values,
                worker_count,
                0_u64,
                |sum, value| Ok::<_, ()>(sum + value),
                |left, right| left + right,
            );
            assert_eq!(result, Ok(expected), "worker_count={worker_count}");
        }
    }
}

#[test]
fn parallel_iteration_produces_the_same_indexed_result_on_repeated_runs() {
    let expected: Vec<u32> = (0..257).map(|value| value * 3 + 1).collect();

    for worker_count in [1, 2, 3, 7, 64] {
        for _ in 0..16 {
            let mut values: Vec<u32> = (0..257).collect();
            for_each_parallel(&mut values, worker_count, |value| {
                *value = *value * 3 + 1;
            })
            .unwrap();
            assert_eq!(values, expected, "worker_count={worker_count}");
        }
    }
}

#[test]
fn parallel_reduction_selects_the_earliest_partition_error_repeatedly() {
    let values: Vec<usize> = (0..128).collect();

    for _ in 0..32 {
        let result = try_reduce_parallel(
            &values,
            8,
            0_usize,
            |sum, value| {
                if *value == 3 || *value == 35 || *value == 99 {
                    Err(*value)
                } else {
                    Ok(sum + value)
                }
            },
            |left, right| left + right,
        );
        assert_eq!(
            result,
            Err(dinah::compute::ParallelReduceError::Operation(3))
        );
    }
}
