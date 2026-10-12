//! Repeatable observable contracts for the public Core API.
use dinah::core::{CoreConfig, CoreId, LifecycleState, Validate};

#[test]
fn configuration_validation_and_errors_are_repeatable() {
    for _ in 0..128 {
        assert_eq!(CoreConfig::default(), CoreConfig::new(1, 64).unwrap());
        assert_eq!(
            CoreConfig::new(0, 1),
            Err(dinah::core::CoreConfigError::ZeroWorkers)
        );
        assert_eq!(
            CoreConfig::new(1, 0),
            Err(dinah::core::CoreConfigError::ZeroBatchSize)
        );
        assert_eq!(
            CoreConfig::new(0, 0),
            Err(dinah::core::CoreConfigError::ZeroWorkers)
        );
        let config = CoreConfig::new(usize::MAX, usize::MAX).unwrap();
        assert_eq!(config.validate(), Ok(()));
        assert_eq!(Validate::validate(&config), Ok(()));
    }
}

#[test]
fn caller_supplied_ids_keep_stable_value_order_and_format() {
    for value in [1, 2, 42, u32::MAX as u64, u64::MAX] {
        for _ in 0..128 {
            let id = CoreId::new(value).unwrap();
            assert_eq!(id.get(), value);
            assert_eq!(id.to_string(), value.to_string());
            assert_eq!(CoreId::new(id.get()), Ok(id));
        }
    }
    assert!(CoreId::new(0).is_err());
    assert!(CoreId::new(0).is_err());
}

#[test]
fn lifecycle_transition_predicates_are_repeatable_and_side_effect_free() {
    use LifecycleState::{Ready, Running, Stopped, Stopping};
    let states = [Ready, Running, Stopping, Stopped];
    let expected = [
        [false, true, false, true],
        [false, false, true, false],
        [false, false, false, true],
        [false, false, false, false],
    ];

    for _ in 0..128 {
        for (from_index, from) in states.iter().copied().enumerate() {
            for (to_index, to) in states.iter().copied().enumerate() {
                assert_eq!(
                    from.can_transition_to(to),
                    expected[from_index][to_index],
                    "transition {from:?} -> {to:?}"
                );
            }
        }
    }
}
