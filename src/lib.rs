//! # State Tutorial
//!
//! Implementation of an ESSOR/SCA-inspired component lifecycle state machine.
//!
//! ## State Machine Diagram
//!
//! ```text
//!
//!                          ┌───────────────────────────────────────┐
//!                          │              reset()                  │
//!                          │         ┌──────────────────────────┐  │
//!                          │         │        reset()           │  │
//!                          ▼         │    ┌──────────────────┐  │  │
//!                    ┌──────────┐    │    │     reset()      │  │  │
//!             ──────►│ INACTIVE │    │    │  ┌───────────┐   │  │  │
//!                    └──────────┘    │    │  │           │   │  │  │
//!                          │         │    │  │   reset() │   │  │  │
//!                     config()       │    │  │    ┌──────┴───┴──┴──┴─┐
//!                          │         │    │  │    │      ERROR       │
//!                          ▼         │    │  │    └──────────────────┘
//!                    ┌──────────┐    │    │  │           ▲
//!                    │  LOADED  │────┘    │  │        (any failure)
//!                    └──────────┘         │  │
//!                          │              │  │
//!                     config()            │  │
//!                          │              │  │
//!                          ▼              │  │
//!                    ┌──────────┐─────────┘  │
//!                    │  READY   │            │
//!                    └──────────┘────────────┘
//!                          │         ▲
//!                     start()     stop()
//!                          │         │
//!                          ▼         │
//!                    ┌──────────┐    │
//!                    │ RUNNING  │────┘
//!                    └──────────┘
//!                          │
//!                       reset()
//!                          │
//!                          ▼
//!                    ┌──────────┐
//!                    │ INACTIVE │  (see top)
//!                    └──────────┘
//!
//! ```
//!
//! ## Valid Transitions
//!
//! | Current State | Method     | Next State | Notes                        |
//! |---------------|------------|------------|------------------------------|
//! | Inactive      | config()   | Loaded     | Load & first configuration   |
//! | Loaded        | config()   | Ready      | Full configuration           |
//! | Ready         | start()    | Running    | Activate the component       |
//! | Running       | stop()     | Ready      | Deactivate the component     |
//! | Any           | reset()    | Inactive   | Hard reset to initial state  |
//! | Loaded        | test()     | Loaded     | Self-test (loaded state)     |
//! | Ready         | test()     | Ready      | Self-test (ready state)      |
//! | Any           | query()    | —          | Read-only, no state change   |
//! | Any (failure) | *any*      | Error      | Transition on error          |

use std::fmt;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Possible states of an ESSOR/SCA-inspired component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentState {
    /// Component exists but has not been loaded or configured.
    Inactive,
    /// Component has been loaded with initial configuration.
    Loaded,
    /// Component is fully configured and ready to be started.
    Ready,
    /// Component is actively running.
    Running,
    /// Component encountered an error; must be reset before reuse.
    Error(String),
}

impl fmt::Display for ComponentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentState::Inactive => write!(f, "INACTIVE"),
            ComponentState::Loaded => write!(f, "LOADED"),
            ComponentState::Ready => write!(f, "READY"),
            ComponentState::Running => write!(f, "RUNNING"),
            ComponentState::Error(msg) => write!(f, "ERROR({})", msg),
        }
    }
}

/// Errors returned by component operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentError {
    /// The requested method is not valid in the current state.
    InvalidTransition {
        current: ComponentState,
        operation: &'static str,
    },
    /// An internal error occurred during the operation.
    InternalError(String),
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::InvalidTransition { current, operation } => {
                write!(
                    f,
                    "Cannot call '{}' while in state '{}'",
                    operation, current
                )
            }
            ComponentError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

/// Configuration parameters passed to `config()`.
#[derive(Debug, Clone, Default)]
pub struct ConfigParams {
    /// Arbitrary key–value pairs for component configuration.
    pub entries: Vec<(String, String)>,
}

impl ConfigParams {
    /// Create a new, empty [`ConfigParams`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a configuration entry.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.entries.push((key.into(), value.into()));
        self
    }
}

/// Status snapshot returned by `query()`.
#[derive(Debug, Clone)]
pub struct ComponentStatus {
    pub state: ComponentState,
    pub description: String,
}

/// Result returned by `test()`.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub details: String,
}

// ---------------------------------------------------------------------------
// Component interface (trait)
// ---------------------------------------------------------------------------

/// Interface that every ESSOR/SCA-inspired component must implement.
///
/// Each method drives the internal state machine. Calling a method that is
/// not valid in the current state returns a [`ComponentError::InvalidTransition`].
pub trait ComponentInterface {
    /// Start the component (Ready → Running).
    fn start(&mut self) -> Result<(), ComponentError>;

    /// Stop the component (Running → Ready).
    fn stop(&mut self) -> Result<(), ComponentError>;

    /// Reset the component to its initial state (any → Inactive).
    fn reset(&mut self) -> Result<(), ComponentError>;

    /// Query the current status of the component (read-only).
    fn query(&self) -> ComponentStatus;

    /// Apply configuration parameters (Inactive → Loaded, or Loaded → Ready).
    fn config(&mut self, params: ConfigParams) -> Result<(), ComponentError>;

    /// Run the built-in self-test (valid in Loaded or Ready states).
    fn test(&mut self) -> Result<TestResult, ComponentError>;
}

// ---------------------------------------------------------------------------
// Concrete component implementation
// ---------------------------------------------------------------------------

/// A concrete component that implements the ESSOR/SCA-inspired lifecycle.
pub struct Component {
    name: String,
    state: ComponentState,
    config: Vec<(String, String)>,
}

impl Component {
    /// Create a new component in the [`ComponentState::Inactive`] state.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: ComponentState::Inactive,
            config: Vec::new(),
        }
    }

    /// Return a reference to the current state.
    pub fn state(&self) -> &ComponentState {
        &self.state
    }

    /// Return the component name.
    pub fn name(&self) -> &str {
        &self.name
    }

    // Internal helper: transition to Error and return an Err.
    fn fail(&mut self, msg: impl Into<String>) -> ComponentError {
        let msg = msg.into();
        self.state = ComponentState::Error(msg.clone());
        ComponentError::InternalError(msg)
    }
}

impl ComponentInterface for Component {
    /// Start the component.
    ///
    /// Valid only when the component is in [`ComponentState::Ready`].
    /// Transitions to [`ComponentState::Running`].
    fn start(&mut self) -> Result<(), ComponentError> {
        match &self.state {
            ComponentState::Ready => {
                self.state = ComponentState::Running;
                Ok(())
            }
            other => Err(ComponentError::InvalidTransition {
                current: other.clone(),
                operation: "start",
            }),
        }
    }

    /// Stop the component.
    ///
    /// Valid only when the component is in [`ComponentState::Running`].
    /// Transitions to [`ComponentState::Ready`].
    fn stop(&mut self) -> Result<(), ComponentError> {
        match &self.state {
            ComponentState::Running => {
                self.state = ComponentState::Ready;
                Ok(())
            }
            other => Err(ComponentError::InvalidTransition {
                current: other.clone(),
                operation: "stop",
            }),
        }
    }

    /// Reset the component to the initial state.
    ///
    /// Valid from any state (including [`ComponentState::Error`]).
    /// Always transitions to [`ComponentState::Inactive`].
    fn reset(&mut self) -> Result<(), ComponentError> {
        self.config.clear();
        self.state = ComponentState::Inactive;
        Ok(())
    }

    /// Query the component's current status.
    ///
    /// This is a read-only operation; the state is never modified.
    fn query(&self) -> ComponentStatus {
        let description = match &self.state {
            ComponentState::Inactive => {
                format!("Component '{}' is inactive.", self.name)
            }
            ComponentState::Loaded => {
                format!(
                    "Component '{}' is loaded with {} configuration entries.",
                    self.name,
                    self.config.len()
                )
            }
            ComponentState::Ready => {
                format!(
                    "Component '{}' is fully configured and ready to start.",
                    self.name
                )
            }
            ComponentState::Running => {
                format!("Component '{}' is running.", self.name)
            }
            ComponentState::Error(msg) => {
                format!("Component '{}' is in error state: {}", self.name, msg)
            }
        };
        ComponentStatus {
            state: self.state.clone(),
            description,
        }
    }

    /// Configure the component.
    ///
    /// - [`ComponentState::Inactive`] → [`ComponentState::Loaded`]:  
    ///   Loads initial configuration parameters.
    /// - [`ComponentState::Loaded`] → [`ComponentState::Ready`]:  
    ///   Applies final configuration; component is ready to start.
    ///
    /// Calling `config()` in any other state is an invalid transition.
    fn config(&mut self, params: ConfigParams) -> Result<(), ComponentError> {
        match &self.state {
            ComponentState::Inactive => {
                self.config = params.entries;
                self.state = ComponentState::Loaded;
                Ok(())
            }
            ComponentState::Loaded => {
                // Merge additional parameters.
                self.config.extend(params.entries);
                self.state = ComponentState::Ready;
                Ok(())
            }
            other => Err(ComponentError::InvalidTransition {
                current: other.clone(),
                operation: "config",
            }),
        }
    }

    /// Run the built-in self-test.
    ///
    /// Valid only in [`ComponentState::Loaded`] or [`ComponentState::Ready`].
    /// On success the state is unchanged; on failure the component transitions
    /// to [`ComponentState::Error`].
    fn test(&mut self) -> Result<TestResult, ComponentError> {
        match &self.state {
            ComponentState::Loaded | ComponentState::Ready => {
                // Simulate a self-test: verify that configuration is non-empty.
                if self.config.is_empty() {
                    let err = self.fail("self-test failed: no configuration entries found");
                    return Err(err);
                }
                Ok(TestResult {
                    passed: true,
                    details: format!(
                        "Self-test passed: {} configuration entries verified.",
                        self.config.len()
                    ),
                })
            }
            other => Err(ComponentError::InvalidTransition {
                current: other.clone(),
                operation: "test",
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helpers -----------------------------------------------------------

    fn new_component() -> Component {
        Component::new("test-component")
    }

    fn basic_params() -> ConfigParams {
        ConfigParams::new().with("frequency", "100MHz")
    }

    // --- Initial state -----------------------------------------------------

    #[test]
    fn initial_state_is_inactive() {
        let c = new_component();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    // --- config() ----------------------------------------------------------

    #[test]
    fn config_inactive_transitions_to_loaded() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        assert_eq!(*c.state(), ComponentState::Loaded);
    }

    #[test]
    fn config_loaded_transitions_to_ready() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(ConfigParams::new().with("mode", "active")).unwrap();
        assert_eq!(*c.state(), ComponentState::Ready);
    }

    #[test]
    fn config_ready_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap(); // now Ready
        let err = c.config(basic_params()).unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "config",
                ..
            }
        ));
    }

    #[test]
    fn config_running_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        let err = c.config(basic_params()).unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "config",
                ..
            }
        ));
    }

    // --- start() -----------------------------------------------------------

    #[test]
    fn start_ready_transitions_to_running() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        assert_eq!(*c.state(), ComponentState::Running);
    }

    #[test]
    fn start_inactive_returns_invalid_transition() {
        let mut c = new_component();
        let err = c.start().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "start",
                ..
            }
        ));
    }

    #[test]
    fn start_loaded_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        let err = c.start().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "start",
                ..
            }
        ));
    }

    #[test]
    fn start_running_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        let err = c.start().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "start",
                ..
            }
        ));
    }

    // --- stop() ------------------------------------------------------------

    #[test]
    fn stop_running_transitions_to_ready() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        c.stop().unwrap();
        assert_eq!(*c.state(), ComponentState::Ready);
    }

    #[test]
    fn stop_ready_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        let err = c.stop().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "stop",
                ..
            }
        ));
    }

    #[test]
    fn stop_inactive_returns_invalid_transition() {
        let mut c = new_component();
        let err = c.stop().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "stop",
                ..
            }
        ));
    }

    // --- reset() -----------------------------------------------------------

    #[test]
    fn reset_from_inactive_stays_inactive() {
        let mut c = new_component();
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    #[test]
    fn reset_from_loaded_transitions_to_inactive() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    #[test]
    fn reset_from_ready_transitions_to_inactive() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    #[test]
    fn reset_from_running_transitions_to_inactive() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    #[test]
    fn reset_from_error_transitions_to_inactive() {
        let mut c = new_component();
        // Trigger error: test with empty config
        c.config(ConfigParams::new()).unwrap(); // Inactive → Loaded (empty config)
        let _ = c.test(); // Should fail → Error
        assert!(matches!(*c.state(), ComponentState::Error(_)));
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    // --- query() -----------------------------------------------------------

    #[test]
    fn query_does_not_change_state() {
        let mut c = new_component();
        let status = c.query();
        assert_eq!(status.state, ComponentState::Inactive);
        assert_eq!(*c.state(), ComponentState::Inactive);

        c.config(basic_params()).unwrap();
        let status = c.query();
        assert_eq!(status.state, ComponentState::Loaded);
        assert_eq!(*c.state(), ComponentState::Loaded);
    }

    #[test]
    fn query_returns_description_for_each_state() {
        let mut c = new_component();

        // Inactive
        let s = c.query();
        assert!(s.description.contains("inactive"));

        // Loaded
        c.config(basic_params()).unwrap();
        let s = c.query();
        assert!(s.description.contains("loaded"));

        // Ready
        c.config(basic_params()).unwrap();
        let s = c.query();
        assert!(s.description.contains("ready"));

        // Running
        c.start().unwrap();
        let s = c.query();
        assert!(s.description.contains("running"));
    }

    // --- test() ------------------------------------------------------------

    #[test]
    fn test_in_loaded_state_passes_and_stays_loaded() {
        let mut c = new_component();
        c.config(basic_params()).unwrap(); // → Loaded
        let result = c.test().unwrap();
        assert!(result.passed);
        assert_eq!(*c.state(), ComponentState::Loaded);
    }

    #[test]
    fn test_in_ready_state_passes_and_stays_ready() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap(); // → Ready
        let result = c.test().unwrap();
        assert!(result.passed);
        assert_eq!(*c.state(), ComponentState::Ready);
    }

    #[test]
    fn test_in_inactive_returns_invalid_transition() {
        let mut c = new_component();
        let err = c.test().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "test",
                ..
            }
        ));
    }

    #[test]
    fn test_in_running_returns_invalid_transition() {
        let mut c = new_component();
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        let err = c.test().unwrap_err();
        assert!(matches!(
            err,
            ComponentError::InvalidTransition {
                operation: "test",
                ..
            }
        ));
    }

    #[test]
    fn test_with_empty_config_transitions_to_error() {
        let mut c = new_component();
        c.config(ConfigParams::new()).unwrap(); // Loaded, but config is empty
        let result = c.test();
        assert!(result.is_err());
        assert!(matches!(*c.state(), ComponentState::Error(_)));
    }

    // --- Full lifecycle ----------------------------------------------------

    #[test]
    fn full_lifecycle_inactive_to_running_and_back() {
        let mut c = new_component();

        // Inactive
        assert_eq!(*c.state(), ComponentState::Inactive);

        // config #1: Inactive → Loaded
        c.config(ConfigParams::new().with("param1", "value1"))
            .unwrap();
        assert_eq!(*c.state(), ComponentState::Loaded);

        // config #2: Loaded → Ready
        c.config(ConfigParams::new().with("param2", "value2"))
            .unwrap();
        assert_eq!(*c.state(), ComponentState::Ready);

        // Self-test in Ready state
        let result = c.test().unwrap();
        assert!(result.passed);
        assert_eq!(*c.state(), ComponentState::Ready);

        // start: Ready → Running
        c.start().unwrap();
        assert_eq!(*c.state(), ComponentState::Running);

        // stop: Running → Ready
        c.stop().unwrap();
        assert_eq!(*c.state(), ComponentState::Ready);

        // reset: Ready → Inactive
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);
    }

    #[test]
    fn error_recovery_via_reset() {
        let mut c = new_component();

        // Reach error state via test with empty config
        c.config(ConfigParams::new()).unwrap();
        let _ = c.test();
        assert!(matches!(*c.state(), ComponentState::Error(_)));

        // Recover via reset
        c.reset().unwrap();
        assert_eq!(*c.state(), ComponentState::Inactive);

        // Normal lifecycle resumes
        c.config(basic_params()).unwrap();
        c.config(basic_params()).unwrap();
        c.start().unwrap();
        assert_eq!(*c.state(), ComponentState::Running);
    }

    // --- Display / Debug ---------------------------------------------------

    #[test]
    fn component_state_display() {
        assert_eq!(ComponentState::Inactive.to_string(), "INACTIVE");
        assert_eq!(ComponentState::Loaded.to_string(), "LOADED");
        assert_eq!(ComponentState::Ready.to_string(), "READY");
        assert_eq!(ComponentState::Running.to_string(), "RUNNING");
        assert_eq!(
            ComponentState::Error("boom".to_string()).to_string(),
            "ERROR(boom)"
        );
    }

    #[test]
    fn component_error_display() {
        let err = ComponentError::InvalidTransition {
            current: ComponentState::Inactive,
            operation: "start",
        };
        assert!(err.to_string().contains("start"));
        assert!(err.to_string().contains("INACTIVE"));

        let err2 = ComponentError::InternalError("oops".to_string());
        assert!(err2.to_string().contains("oops"));
    }
}
