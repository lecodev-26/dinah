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
