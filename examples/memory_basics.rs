//! A bounded Buffer example showing capacity and rejected insertion behavior.

use dinah::memory::{Buffer, BufferPushError};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut samples = Buffer::try_with_capacity(3)?;
    samples.try_extend_from_slice(&[12_u16, 18])?;
    samples
        .try_push(24)
        .expect("the configured capacity has one free slot");

    println!(
        "samples={:?} len={} capacity={} full={}",
        samples.as_slice(),
        samples.len(),
        samples.capacity(),
        samples.is_full()
    );

    // A full buffer returns the rejected value instead of growing implicitly.
    match samples.try_push(30) {
        Ok(()) => unreachable!("the configured capacity is already full"),
        Err(BufferPushError::Full(value)) => println!("rejected={value}"),
    }

    samples.clear(); // drops current values while retaining the reserved storage
    samples.try_push(42)?;
    println!("after_clear={:?}", samples.as_slice());
    Ok(())
}
