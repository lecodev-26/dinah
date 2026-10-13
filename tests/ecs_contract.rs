//! Public ECS contracts that downstream users can rely on.
use dinah::ecs::Component;
use std::cell::Cell;
use std::rc::Rc;

fn assert_component<T: Component>() {}

#[test]
fn component_is_implemented_for_static_types_without_user_boilerplate() {
    assert_component::<u32>();
    assert_component::<String>();
    assert_component::<()>();
}

#[test]
fn initial_component_contract_does_not_require_send_or_sync() {
    // Rc<Cell<_>> is neither Send nor Sync. This is intentional while the ECS
    // World is single-owner and has no implicit concurrent access.
    assert_component::<Rc<Cell<u32>>>();
}

#[test]
fn public_world_allocates_reuses_and_rejects_stale_entities() {
    use dinah::ecs::{World, WorldError};

    let mut world = World::new();
    let first = world.spawn().unwrap();
    assert!(world.contains(first));
    assert_eq!(world.len(), 1);
    world.despawn(first).unwrap();
    assert!(!world.contains(first));
    assert_eq!(world.despawn(first), Err(WorldError::EntityNotAlive));

    let replacement = world.spawn().unwrap();
    assert_eq!(replacement.index(), first.index());
    assert_ne!(replacement.generation(), first.generation());
    assert!(!world.contains(first));
    assert!(world.contains(replacement));
}

#[test]
fn public_world_tracks_empty_state_and_multiple_entities() {
    use dinah::ecs::World;

    let mut world = World::default();
    assert!(world.is_empty());
    let a = world.spawn().unwrap();
    let b = world.spawn().unwrap();
    assert_ne!(a, b);
    assert_eq!(world.len(), 2);
    world.despawn(a).unwrap();
    assert_eq!(world.len(), 1);
    assert!(!world.is_empty());
    world.despawn(b).unwrap();
    assert!(world.is_empty());
}

#[test]
fn public_world_runs_explicit_systems_through_runtime() {
    use dinah::ecs::World;
    use dinah::runtime::Scheduler;

    let scheduler = Scheduler::new();
    let mut world = World::new();
    let entity = world
        .run_system(&scheduler, |world| {
            world.spawn().map_err(|_| "spawn failed")
        })
        .expect("the synchronous scheduler accepts the system")
        .expect("system should spawn an entity");
    assert!(world.contains(entity));

    let failure = world.run_system(&scheduler, |_| Err::<(), _>("expected failure"));
    assert_eq!(failure, Ok(Err("expected failure")));
    assert_eq!(world.len(), 1);
}

#[test]
fn public_system_access_metadata_detects_component_conflicts() {
    use dinah::ecs::SystemAccess;

    struct Position;
    struct Velocity;

    let mut reader_a = SystemAccess::new();
    reader_a.read::<Position>();
    let mut reader_b = SystemAccess::new();
    reader_b.read::<Position>();
    assert!(!reader_a.conflicts_with(&reader_b));

    let mut writer = SystemAccess::new();
    writer.write::<Position>();
    assert!(reader_a.conflicts_with(&writer));
    assert!(writer.conflicts_with(&reader_b));

    let mut other_writer = SystemAccess::new();
    other_writer.write::<Velocity>();
    assert!(!writer.conflicts_with(&other_writer));
}

#[test]
fn public_system_access_metadata_does_not_require_send_or_sync() {
    use dinah::ecs::SystemAccess;

    let mut local_access = SystemAccess::new();
    local_access.write::<Rc<Cell<u32>>>();
    assert!(!local_access.is_empty());
}

#[test]
fn public_system_plan_groups_compatible_accesses_deterministically() {
    use dinah::ecs::{SystemAccess, SystemPlan};

    struct Position;
    struct Velocity;

    let mut first = SystemAccess::new();
    first.read::<Position>();
    let mut second = SystemAccess::new();
    second.read::<Position>().write::<Velocity>();
    let mut third = SystemAccess::new();
    third.write::<Position>();

    let accesses = [first, second, third];
    let expected = SystemPlan::from_accesses(&accesses);
    assert_eq!(expected.batches(), &[vec![0, 1], vec![2]]);
    for _ in 0..32 {
        assert_eq!(SystemPlan::from_accesses(&accesses), expected);
    }
}

#[test]
fn public_system_plan_keeps_conflicts_ordered_and_groups_compatible_systems() {
    use dinah::ecs::{SystemAccess, SystemPlan};

    struct Position;
    struct Velocity;

    let mut first = SystemAccess::new();
    first.write::<Position>();
    let mut second = SystemAccess::new();
    second.write::<Velocity>();
    let mut third = SystemAccess::new();
    third.read::<Position>();
    let mut fourth = SystemAccess::new();
    fourth.write::<Velocity>();

    let plan = SystemPlan::from_accesses(&[first, second, third, fourth]);
    assert_eq!(plan.batches(), &[vec![0, 1], vec![2, 3]]);
}

#[test]
fn public_world_manages_component_lifecycle_and_despawn_cleanup() {
    use dinah::ecs::{World, WorldError};

    let mut world = World::new();
    let entity = world.spawn().unwrap();
    assert_eq!(world.insert(entity, 12_u32), Ok(None));
    assert_eq!(world.get::<u32>(entity), Some(&12));
    assert!(world.has::<u32>(entity));
    assert_eq!(world.insert(entity, 13_u32), Ok(Some(12)));
    *world.get_mut::<u32>(entity).unwrap() += 1;
    assert_eq!(world.get::<u32>(entity), Some(&14));
    assert_eq!(world.remove::<u32>(entity), Some(14));
    assert!(!world.has::<u32>(entity));

    world
        .insert(entity, String::from("removed on despawn"))
        .unwrap();
    world.despawn(entity).unwrap();
    assert_eq!(world.get::<String>(entity), None);
    assert_eq!(
        world.insert(entity, 99_u32),
        Err(WorldError::EntityNotAlive)
    );

    let replacement = world.spawn().unwrap();
    assert_eq!(replacement.index(), entity.index());
    assert!(!world.has::<String>(replacement));
}

#[test]
fn public_world_components_support_non_send_values() {
    use dinah::ecs::World;

    let mut world = World::new();
    let entity = world.spawn().unwrap();
    let local = Rc::new(Cell::new(3_u32));
    world.insert(entity, Rc::clone(&local)).unwrap();
    world.get::<Rc<Cell<u32>>>(entity).unwrap().set(8);
    assert_eq!(local.get(), 8);
}
