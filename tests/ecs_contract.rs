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
