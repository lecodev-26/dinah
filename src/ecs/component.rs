//! Component marker and its initial type contract.

/// A value that can be stored as an ECS component.
///
/// The initial ECS accepts every `'static` Rust type. This keeps components
/// ergonomic and permits types that are not `Send` or `Sync` while `World` is
/// single-owner. The bounds can be narrowed only if a future concurrent API
/// requires it. The blanket implementation means user-defined marker impls
/// are unnecessary.
pub trait Component: 'static {}

impl<T: 'static> Component for T {}

#[cfg(test)]
mod tests {
    use super::Component;
    use std::cell::Cell;
    use std::rc::Rc;

    fn assert_component<T: Component>() {}

    #[test]
    fn static_types_are_components_without_manual_marker_impls() {
        assert_component::<u32>();
        assert_component::<()>();
        assert_component::<String>();
        assert_component::<Rc<Cell<u32>>>();
    }
}
