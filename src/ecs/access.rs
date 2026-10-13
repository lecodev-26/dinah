//! Typed component-access declarations for ECS systems.
use super::Component;
use core::any::TypeId;

/// Access mode a system declares for one component type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    /// The system only reads this component type.
    Read,
    /// The system may mutate this component type.
    Write,
}

/// Explicit component access metadata for one system.
///
/// This descriptor is planning metadata only: it does not borrow a `World`,
/// validate entity liveness, or execute a system. Repeated declarations of a
/// type are merged conservatively, so any write declaration wins over reads.
#[derive(Debug, Clone, Default)]
pub struct SystemAccess {
    entries: Vec<(TypeId, AccessMode)>,
}

impl SystemAccess {
    /// Creates an empty access declaration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Declares read-only access to component type `T`.
    pub fn read<T: Component>(&mut self) -> &mut Self {
        self.declare::<T>(AccessMode::Read)
    }

    /// Declares potentially mutable access to component type `T`.
    pub fn write<T: Component>(&mut self) -> &mut Self {
        self.declare::<T>(AccessMode::Write)
    }

    fn declare<T: Component>(&mut self, mode: AccessMode) -> &mut Self {
        let type_id = TypeId::of::<T>();
        if let Some((_, current)) = self.entries.iter_mut().find(|(id, _)| *id == type_id) {
            if mode == AccessMode::Write {
                *current = AccessMode::Write;
            }
        } else {
            self.entries.push((type_id, mode));
        }
        self
    }

    /// Returns the declared mode for `T`, or `None` if it was not declared.
    #[must_use]
    pub fn mode<T: Component>(&self) -> Option<AccessMode> {
        self.entries
            .iter()
            .find_map(|(id, mode)| (*id == TypeId::of::<T>()).then_some(*mode))
    }

    /// Returns whether this descriptor declares no component accesses.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns whether two systems have a component read/write conflict.
    ///
    /// Read/read access to the same type is compatible. If either system writes
    /// a type read or written by the other, the descriptors conflict. This does
    /// not itself authorize concurrent execution or prove that closures match
    /// their declarations.
    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.entries.iter().any(|(type_id, mode)| {
            let Some(other_mode) = other
                .entries
                .iter()
                .find_map(|(other_id, other_mode)| (other_id == type_id).then_some(other_mode))
            else {
                return false;
            };
            *mode == AccessMode::Write || *other_mode == AccessMode::Write
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AccessMode, SystemAccess};
    use std::cell::Cell;
    use std::rc::Rc;

    struct Position;
    struct Velocity;

    #[test]
    fn read_read_of_same_type_is_compatible() {
        let mut first = SystemAccess::new();
        first.read::<Position>();
        let mut second = SystemAccess::new();
        second.read::<Position>();
        assert!(!first.conflicts_with(&second));
    }

    #[test]
    fn read_write_and_write_write_of_same_type_conflict() {
        let mut reader = SystemAccess::new();
        reader.read::<Position>();
        let mut writer = SystemAccess::new();
        writer.write::<Position>();
        assert!(reader.conflicts_with(&writer));
        assert!(writer.conflicts_with(&reader));
        assert!(writer.conflicts_with(&writer));
    }

    #[test]
    fn distinct_types_do_not_conflict() {
        let mut position = SystemAccess::new();
        position.write::<Position>();
        let mut velocity = SystemAccess::new();
        velocity.write::<Velocity>();
        assert!(!position.conflicts_with(&velocity));
    }

    #[test]
    fn repeated_declarations_are_deduplicated_and_write_wins() {
        let mut access = SystemAccess::new();
        access.read::<Position>().read::<Position>();
        assert_eq!(access.mode::<Position>(), Some(AccessMode::Read));
        access.write::<Position>().read::<Position>();
        assert_eq!(access.mode::<Position>(), Some(AccessMode::Write));
        assert!(access.conflicts_with(&{
            let mut reader = SystemAccess::new();
            reader.read::<Position>();
            reader
        }));
    }

    #[test]
    fn empty_access_sets_are_compatible_and_non_send_components_are_describable() {
        let empty = SystemAccess::new();
        assert!(empty.is_empty());
        assert!(!empty.conflicts_with(&empty));
        let mut local_only = SystemAccess::new();
        local_only.write::<Rc<Cell<u32>>>();
        assert_eq!(local_only.mode::<Rc<Cell<u32>>>(), Some(AccessMode::Write));
    }
}
