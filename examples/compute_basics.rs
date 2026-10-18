//! Small examples of sequential and explicitly parallel Compute operations.

use dinah::compute::{for_each_batch, for_each_parallel, map, try_reduce};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let input = [1_u32, 2, 3, 4, 5, 6];

    // `map` preserves input order and returns a new vector.
    let squares = map(&input, |value| value * value);
    println!("squares={squares:?}");

    // Batch operations mutate the original slice without creating batch buffers.
    let mut percentages = [10_u32, 20, 30, 40, 50];
    for_each_batch(&mut percentages, 2, |batch| {
        for value in batch {
            *value += 1;
        }
    })?;
    println!("batched={percentages:?}");

    // Parallel execution is opt-in; it does not guarantee a speedup for small inputs.
    let mut doubled = input;
    for_each_parallel(&mut doubled, 2, |value| *value *= 2)?;
    println!("parallel={doubled:?}");

    let sum = try_reduce(&input, 0_u32, |total, value| {
        Ok::<_, std::convert::Infallible>(total + value)
    })?;
    println!("sum={sum}");
    Ok(())
}
