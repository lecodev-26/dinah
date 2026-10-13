//! Minimal deterministic Simulation and SimulationEntities example.

use std::error::Error;
use std::time::Duration;

use dinah::simulation::{Simulation, SimulationEntities};

#[derive(Debug)]
struct Particle {
    position: f64,
    velocity: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Time advances only when step succeeds; no wall clock is consulted.
    let initial = vec![Particle {
        position: 0.0,
        velocity: 2.0,
    }];
    let mut simulation = Simulation::new(initial, Duration::from_millis(16))?;

    for _ in 0..3 {
        let time = simulation.step(|particles, time| {
            let delta_seconds = time.delta.as_secs_f64();
            for particle in particles {
                particle.position += particle.velocity * delta_seconds;
            }
            Ok::<(), std::convert::Infallible>(())
        })?;
        println!(
            "tick={} elapsed_ms={} position={:.3}",
            time.tick,
            time.elapsed.as_millis(),
            simulation.state()[0].position
        );
    }

    // SimulationEntities owns ordered domain state and assigns stable local IDs.
    let mut entities = SimulationEntities::new();
    let first = entities.insert(Particle {
        position: 10.0,
        velocity: -1.0,
    })?;
    let second = entities.insert(Particle {
        position: 20.0,
        velocity: 0.5,
    })?;

    entities.for_each_mut(|_id, particle| {
        particle.position += particle.velocity;
    });

    for (id, particle) in entities.iter() {
        println!("entity={id:?} position={:.3}", particle.position);
    }

    assert!(entities.get(first).is_some());
    assert!(entities.get(second).is_some());
    Ok(())
}
