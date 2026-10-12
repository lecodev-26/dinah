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

#[test]
fn sequential_compute_contracts_repeat_order_errors_and_empty_inputs() {
    use dinah::compute::{for_each, for_each_batch, map, try_reduce, BatchError};

    for repetition in 0..64 {
        let mut values = [3_i32, 1, 4, 1, 5];
        let mut visit_order = Vec::new();
        for_each(&mut values, |value| {
            visit_order.push(*value);
            *value += repetition;
        });
        assert_eq!(visit_order, [3, 1, 4, 1, 5]);
        assert_eq!(
            values,
            [
                3 + repetition,
                1 + repetition,
                4 + repetition,
                1 + repetition,
                5 + repetition
            ]
        );

        let mapped = map(&values, |value| value * 2);
        assert_eq!(mapped, values.map(|value| value * 2));

        let mut batched = values;
        let mut batches = Vec::new();
        for_each_batch(&mut batched, 2, |batch| {
            batches.push(batch.len());
            batch.reverse();
        })
        .unwrap();
        assert_eq!(batches, [2, 2, 1]);
        assert_eq!(
            batched,
            [values[1], values[0], values[3], values[2], values[4]]
        );

        let mut unchanged = values;
        assert_eq!(
            for_each_batch(&mut unchanged, 0, |_| panic!(
                "invalid batch must not call closure"
            )),
            Err(BatchError::ZeroSize)
        );
        assert_eq!(unchanged, values);

        let mut visited = Vec::new();
        let result = try_reduce(&values, 0_i32, |sum, value| {
            visited.push(*value);
            if *value == values[2] {
                Err(*value)
            } else {
                Ok(sum + value)
            }
        });
        assert_eq!(result, Err(values[2]));
        assert_eq!(visited, values[..=2]);

        assert_eq!(
            try_reduce::<i32, i32, (), _>(&[], 19, |sum, value| Ok(sum + value)),
            Ok(19)
        );
        assert_eq!(map::<i32, i32, _>(&[], |value| *value), Vec::<i32>::new());
    }
}

#[test]
fn pipeline_validation_failure_is_repeatable_and_stops_later_stages() {
    use dinah::compute::{BatchError, ComputePipeline, PipelineError};

    for _ in 0..64 {
        let mut values = [2_i32, 4, 6];
        let mut later_stage_calls = 0;
        let result = ComputePipeline::new(&mut values)
            .for_each(|value| *value += 1)
            .unwrap()
            .for_each_batch(0, |_| {})
            .map(|pipeline| {
                pipeline.for_each(|_| later_stage_calls += 1).unwrap();
            });
        assert_eq!(result, Err(PipelineError::Batch(BatchError::ZeroSize)));
        assert_eq!(later_stage_calls, 0);
        assert_eq!(values, [3, 5, 7]);
    }
}
