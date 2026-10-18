//! Bounded Runtime queue, synchronous dispatch, and explicit parallel batches.
use dinah::runtime::{Scheduler, TaskQueue};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = Scheduler::new();
    let mut queue = TaskQueue::new(NonZeroUsize::new(3).ok_or("capacity must be positive")?);
    for value in 1_u32..=3 {
        queue
            .try_push(move || Ok::<_, &'static str>(value * value))
            .map_err(|_| "queue unexpectedly full")?;
    }

    // Results retain FIFO dispatch order; the scheduler runs this queue explicitly.
    let sequential = scheduler.run_until_empty(&mut queue);
    println!("sequential={sequential:?}");

    let values = Arc::new(Mutex::new(Vec::new()));
    let mut parallel = TaskQueue::new(NonZeroUsize::new(4).ok_or("capacity must be positive")?);
    for value in 1_u32..=4 {
        let values = Arc::clone(&values);
        parallel
            .try_push(move || {
                values
                    .lock()
                    .map_err(|_| "result buffer poisoned")?
                    .push(value);
                Ok::<_, &'static str>(value * 10)
            })
            .map_err(|_| "queue unexpectedly full")?;
    }
    let results = scheduler.run_parallel(
        &mut parallel,
        NonZeroUsize::new(2).ok_or("workers must be positive")?,
    );
    println!("parallel_results={results:?}");
    let mut observed = values.lock().map_err(|_| "result buffer poisoned")?.clone();
    observed.sort_unstable();
    println!("observed_values={observed:?}");
    Ok(())
}
