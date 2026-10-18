//! Minimal ECS lifecycle: spawn, attach components, update, and despawn.

use dinah::ecs::World;
use dinah::runtime::Scheduler;
use std::error::Error;

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Velocity {
    x: i32,
    y: i32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut world = World::new();
    let entity = world.spawn()?;
    world.insert(entity, Position { x: 2, y: 3 })?;
    world.insert(entity, Velocity { x: 4, y: -1 })?;

    // `run_system` uses the public Runtime adapter. In v0.1 scheduling is synchronous.
    let scheduler = Scheduler::new();
    world.run_system(&scheduler, |world| {
        let velocity = world
            .get::<Velocity>(entity)
            .map(|v| (v.x, v.y))
            .ok_or("velocity component is missing")?;
        let position = world
            .get_mut::<Position>(entity)
            .ok_or("position component is missing")?;
        position.x += velocity.0;
        position.y += velocity.1;
        Ok::<_, &'static str>(())
    })??;

    let position = world.get::<Position>(entity).ok_or("position is missing")?;
    println!("entity={entity} position=({}, {})", position.x, position.y);
    world.despawn(entity)?;
    assert!(!world.contains(entity));
    Ok(())
}
