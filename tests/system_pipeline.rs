//! Cross-subsystem smoke test for the public DINAH API.
use dinah::compute::map;
use dinah::core::CoreConfig;
use dinah::ecs::World;
use dinah::memory::Buffer;
use dinah::observability::{MetricsRegistry, Subsystem};
use dinah::runtime::Scheduler;
use dinah::simulation::Simulation;
use dinah::storage::runtime::{StorageCommand, StorageWorkQueue};
use dinah::storage::{MemoryStore, Storage};
use std::num::NonZeroUsize;
use std::time::Duration;

#[test]
fn public_subsystems_form_a_deterministic_data_pipeline() {
    let config = CoreConfig::new(2, 8).expect("valid runtime limits");
    assert_eq!(config.max_workers(), 2);
    assert_eq!(config.max_batch_size(), 8);

    let scheduler = Scheduler::new();
    let mut input = Buffer::try_with_capacity(config.max_batch_size()).unwrap();
    input.try_extend_from_slice(&[1_u32, 2, 3, 4]).unwrap();

    // Runtime schedules a Compute operation over the bounded Memory buffer.
    let transformed = scheduler
        .schedule(|| Ok::<_, &'static str>(map(input.as_slice(), |value| value * 10)))
        .expect("scheduler accepts the operation")
        .expect("compute operation succeeds");
    assert_eq!(transformed, [10, 20, 30, 40]);

    // ECS owns the computed data, and Simulation advances it by fixed logical time.
    let mut world = World::new();
    let entity = world.spawn().expect("entity allocation succeeds");
    world.insert(entity, transformed).unwrap();
    let mut simulation = Simulation::new(world, Duration::from_millis(16)).unwrap();
    let time = simulation
        .step(|world, time| {
            let values = world
                .get_mut::<Vec<u32>>(entity)
                .ok_or("missing component")?;
            for value in values {
                *value += time.tick as u32;
            }
            Ok::<_, &'static str>(())
        })
        .expect("simulation step succeeds");
    assert_eq!(time.tick, 1);
    assert_eq!(
        simulation.state().get::<Vec<u32>>(entity),
        Some(&vec![11, 21, 31, 41])
    );

    // Runtime's storage queue applies a persisted snapshot in FIFO order.
    let bytes: Vec<u8> = simulation
        .state()
        .get::<Vec<u32>>(entity)
        .unwrap()
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    let mut queue = StorageWorkQueue::new(NonZeroUsize::new(2).unwrap());
    queue
        .enqueue(StorageCommand::Put {
            key: b"simulation/frame-1".to_vec(),
            value: bytes.clone(),
        })
        .unwrap();
    let mut store = MemoryStore::new();
    let storage_result = queue
        .dispatch_next(&scheduler, &mut store)
        .expect("one queued command exists")
        .expect("scheduler dispatch succeeds");
    assert!(storage_result.is_ok());
    assert_eq!(store.get(b"simulation/frame-1"), Some(bytes));

    // Observability is explicitly caller-reported and confirms pipeline stages.
    let metrics = MetricsRegistry::new();
    metrics
        .increment(Subsystem::Runtime, "pipeline.completed", 1)
        .unwrap();
    metrics
        .increment(Subsystem::Compute, "items.transformed", 4)
        .unwrap();
    let exported = metrics.export_json().unwrap();
    assert!(exported.contains("pipeline.completed"));
    assert!(exported.contains("items.transformed"));
}
