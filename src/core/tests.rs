use super::{CoreConfig, CoreConfigError, CoreId, IdError, LifecycleState, CORE_API_VERSION};

#[test]
fn config_defaults_are_valid_and_conservative() {
    let config = CoreConfig::default();
    assert_eq!(config.max_workers(), 1);
    assert_eq!(config.max_batch_size(), 64);
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn config_rejects_zero_limits() {
    assert_eq!(CoreConfig::new(0, 1), Err(CoreConfigError::ZeroWorkers));
    assert_eq!(CoreConfig::new(1, 0), Err(CoreConfigError::ZeroBatchSize));
}

#[test]
fn ids_reject_zero_and_preserve_nonzero_values() {
    assert_eq!(CoreId::new(0), Err(IdError));
    let id = CoreId::new(42).expect("non-zero ID should be accepted");
    assert_eq!(id.get(), 42);
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
fn core_api_version_is_defined() {
    assert_eq!(CORE_API_VERSION, 1);
}
