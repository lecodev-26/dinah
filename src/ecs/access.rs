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

/// A deterministic sequence of compatible system batches.
///
/// This planner preserves declaration order: each system is appended to the
/// current batch if it is compatible with every system already in that batch;
/// otherwise it starts the next batch. It is metadata only and does not execute
/// systems, validate declarations against closures, or account for structural
/// world mutations. Callers must not treat a plan as authorization to access a
/// shared `World` concurrently.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemPlan {
    batches: Vec<Vec<usize>>,
}

impl SystemPlan {
    /// Builds a plan from system access declarations in their required order.
    #[must_use]
    pub fn from_accesses(accesses: &[SystemAccess]) -> Self {
        let mut batches: Vec<Vec<usize>> = Vec::new();
        let mut batch_accesses: Vec<Vec<usize>> = Vec::new();

        for (system_index, access) in accesses.iter().enumerate() {
            let can_append = batch_accesses.last().is_some_and(|indices| {
                indices
                    .iter()
                    .all(|&prior| !access.conflicts_with(&accesses[prior]))
            });

            if can_append {
                let last = batches.len() - 1;
                batches[last].push(system_index);
                batch_accesses[last].push(system_index);
            } else {
                batches.push(vec![system_index]);
                batch_accesses.push(vec![system_index]);
            }
        }

        Self { batches }
    }

    /// Returns the ordered batches of input system indices.
    #[must_use]
    pub fn batches(&self) -> &[Vec<usize>] {
        &self.batches
    }

    /// Returns whether there are no systems to plan.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Returns the number of planned execution phases.
    #[must_use]
    pub fn len(&self) -> usize {
        self.batches.len()
    }
}

#[cfg(test)]
mod plan_tests {
    use super::{SystemAccess, SystemPlan};

    struct Position;
    struct Velocity;

    #[test]
    fn empty_input_produces_no_batches() {
        let plan = SystemPlan::from_accesses(&[]);
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
        assert!(plan.batches().is_empty());
    }

    #[test]
    fn compatible_reads_and_distinct_writes_share_a_batch() {
        let mut first = SystemAccess::new();
        first.read::<Position>();
        let mut second = SystemAccess::new();
        second.read::<Position>().write::<Velocity>();
        let mut third = SystemAccess::new();
        third.write::<Position>();

        let plan = SystemPlan::from_accesses(&[first, second, third]);
        assert_eq!(plan.batches(), &[vec![0, 1], vec![2]]);
    }

    #[test]
    fn conflicting_systems_keep_input_order_across_batches() {
        let mut first = SystemAccess::new();
        first.write::<Position>();
        let mut independent = SystemAccess::new();
        independent.write::<Velocity>();
        let mut third = SystemAccess::new();
        third.read::<Position>();
        let mut fourth = SystemAccess::new();
        fourth.write::<Velocity>();

        let plan = SystemPlan::from_accesses(&[first, independent, third, fourth]);
        assert_eq!(plan.batches(), &[vec![0, 1], vec![2], vec![3]]);
    }

    #[test]
    fn planning_is_repeatable_and_each_system_appears_once() {
        let mut first = SystemAccess::new();
        first.read::<Position>();
        let mut second = SystemAccess::new();
        second.write::<Position>();
        let mut third = SystemAccess::new();
        third.read::<Velocity>();
        let accesses = [first, second, third];

        let expected = SystemPlan::from_accesses(&accesses);
        for _ in 0..32 {
            assert_eq!(SystemPlan::from_accesses(&accesses), expected);
        }
        let indices: Vec<_> = expected.batches().iter().flatten().copied().collect();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
