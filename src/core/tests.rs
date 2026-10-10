use super::{
    CoreConfig, CoreConfigError, CoreId, IdError, LifecycleState, Validate, CORE_API_VERSION,
    CORE_INVARIANTS,
};

#[test]
fn config_defaults_are_valid_and_conservative() {
    let config = CoreConfig::default();
    assert_eq!(config.max_workers(), 1);
    assert_eq!(config.max_batch_size(), 64);
    assert_eq!(config, CoreConfig::default_config());
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn config_rejects_zero_limits() {
    assert_eq!(CoreConfig::new(0, 1), Err(CoreConfigError::ZeroWorkers));
    assert_eq!(CoreConfig::new(1, 0), Err(CoreConfigError::ZeroBatchSize));
}

#[test]
fn config_reports_worker_error_before_batch_error() {
    assert_eq!(
        CoreConfig::new(0, 0),
        Err(CoreConfigError::ZeroWorkers),
        "when both limits are invalid, worker validation has precedence"
    );
}

#[test]
fn config_accepts_maximum_usize_limits() {
    let config = CoreConfig::new(usize::MAX, usize::MAX)
        .expect("non-zero limits should be accepted without arithmetic");
    assert_eq!(config.max_workers(), usize::MAX);
    assert_eq!(config.max_batch_size(), usize::MAX);
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn ids_reject_zero_and_preserve_nonzero_values() {
    assert_eq!(CoreId::new(0), Err(IdError));
    let id = CoreId::new(42).expect("non-zero ID should be accepted");
    assert_eq!(id.get(), 42);
    assert_eq!(CoreId::new(id.get()), Ok(id));
}

#[test]
fn ids_accept_maximum_u64_and_format_as_decimal() {
    let id = CoreId::new(u64::MAX).expect("maximum non-zero ID should be accepted");
    assert_eq!(id.get(), u64::MAX);
    assert_eq!(id.to_string(), u64::MAX.to_string());
}

#[test]
fn ids_support_ordering_and_hashing_contracts() {
    use std::collections::HashSet;

    let lower = CoreId::new(7).expect("non-zero ID");
    let higher = CoreId::new(8).expect("non-zero ID");
    assert!(lower < higher);

    let mut ids = HashSet::new();
    ids.insert(lower);
    ids.insert(CoreId::new(7).expect("same ID"));
    ids.insert(higher);
    assert_eq!(ids.len(), 2);
}

#[test]
fn lifecycle_transitions_are_explicit() {
    assert!(LifecycleState::Ready.can_transition_to(LifecycleState::Running));
    assert!(LifecycleState::Running.can_transition_to(LifecycleState::Stopping));
    assert!(LifecycleState::Stopping.can_transition_to(LifecycleState::Stopped));
    assert!(!LifecycleState::Stopped.can_transition_to(LifecycleState::Running));
    assert!(!LifecycleState::Running.can_transition_to(LifecycleState::Ready));
}

#[test]
fn every_lifecycle_state_is_copyable_and_hashable() {
    use std::collections::HashSet;
    use LifecycleState::{Ready, Running, Stopped, Stopping};

    let original = Running;
    let copied = original;
    assert_eq!(original, copied);

    let states = [Ready, Running, Stopping, Stopped];
    let unique: HashSet<_> = states.into_iter().collect();
    assert_eq!(unique.len(), 4);
}

#[test]
fn lifecycle_transition_matrix_matches_the_contract() {
    use LifecycleState::{Ready, Running, Stopped, Stopping};

    let cases = [
        (Ready, Ready, false),
        (Ready, Running, true),
        (Ready, Stopping, false),
        (Ready, Stopped, true),
        (Running, Ready, false),
        (Running, Running, false),
        (Running, Stopping, true),
        (Running, Stopped, false),
        (Stopping, Ready, false),
        (Stopping, Running, false),
        (Stopping, Stopping, false),
        (Stopping, Stopped, true),
        (Stopped, Ready, false),
        (Stopped, Running, false),
        (Stopped, Stopping, false),
        (Stopped, Stopped, false),
    ];

    for (current, next, expected) in cases {
        assert_eq!(
            current.can_transition_to(next),
            expected,
            "unexpected transition: {current:?} -> {next:?}"
        );
    }
}

#[test]
fn core_error_conversions_and_messages_are_stable() {
    use super::CoreError;

    assert_eq!(
        CoreConfigError::ZeroWorkers.to_string(),
        "max_workers must be greater than zero"
    );
    assert_eq!(
        CoreConfigError::ZeroBatchSize.to_string(),
        "max_batch_size must be greater than zero"
    );
    assert_eq!(
        CoreError::from(CoreConfigError::ZeroWorkers),
        CoreError::InvalidConfiguration
    );
    assert_eq!(IdError.to_string(), "core identifier must be non-zero");
    assert_eq!(
        CoreError::InvalidState.to_string(),
        "operation is invalid for the current state"
    );
    assert_eq!(CoreError::InvalidId.to_string(), "invalid core identifier");
}

#[test]
fn core_api_version_is_defined() {
    assert_eq!(CORE_API_VERSION, 1);
}

#[test]
fn validate_trait_checks_configuration_contract() {
    let config = CoreConfig::default();
    assert_eq!(Validate::validate(&config), Ok(()));
}

#[test]
fn validate_trait_preserves_specific_configuration_errors() {
    let invalid = CoreConfig::new(0, 1);
    assert_eq!(invalid, Err(CoreConfigError::ZeroWorkers));

    let config = CoreConfig::new(2, 3).expect("valid limits");
    assert_eq!(Validate::validate(&config), Ok(()));
}

#[test]
fn core_invariants_are_documented_in_api() {
    assert_eq!(CORE_INVARIANTS.len(), 4);
    assert!(CORE_INVARIANTS
        .iter()
        .all(|invariant| !invariant.is_empty()));
}
