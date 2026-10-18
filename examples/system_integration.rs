//! Explicit integration of Runtime, Network, Native and observability.
use dinah::native::{Hal, HostPlatform};
use dinah::network::runtime::NetworkWorkQueue;
use dinah::observability::{MetricsRegistry, Subsystem};
use dinah::runtime::Scheduler;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = Scheduler::new();
    let platform = HostPlatform::new();
    let started = Instant::now();
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let output = Arc::clone(&received);
    let mut network = NetworkWorkQueue::<&'static str>::new(NonZeroUsize::new(4).unwrap());
    network
        .enqueue(Box::new(move || {
            output
                .lock()
                .map_err(|_| "received buffer poisoned")?
                .push("hello from explicit network work".to_owned());
            Ok(())
        }))
        .map_err(|_| "network work queue unexpectedly full")?;

    let dispatch = network
        .dispatch_next(&scheduler)
        .ok_or("no queued network work")??;
    dispatch.map_err(|error| format!("network work failed: {error}"))?;
    let metrics = MetricsRegistry::new();
    metrics.increment(Subsystem::Network, "work.dispatched", 1)?;

    println!(
        "host={}/{} pid={}",
        platform.operating_system(),
        platform.architecture(),
        platform.process_id()
    );
    println!(
        "message={}",
        received
            .lock()
            .map_err(|_| "received buffer poisoned")?
            .first()
            .ok_or("no message received")?
    );
    println!("elapsed_us={}", platform.elapsed_since(started).as_micros());
    println!("metrics={}", metrics.export_json()?);
    Ok(())
}
