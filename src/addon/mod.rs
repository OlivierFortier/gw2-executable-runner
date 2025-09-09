/*!
## Error Handling

All errors across Nexus modules use the [NexusError] enum for consistent propagation and logging.
Use the provided `Result<T>` type alias for fallible operations.

## Modules

- [manager]: Executable management logic
- [ui]: UI rendering components
- [init]: Initialization and cleanup routines

*/

pub mod init;
pub mod manager;
pub mod ui;

pub use init::{load, unload};

/// Consistent error types for the nexus addon
#[derive(Debug)]
pub enum NexusError {
    ManagerInitialization(String),
    ProcessLaunch(String),
    ProcessStop(String),
    FileOperation(String),
    ResourceLoading(String),
}

impl std::fmt::Display for NexusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NexusError::ManagerInitialization(msg) => {
                write!(f, "Manager initialization error: {msg}")
            }
            NexusError::ProcessLaunch(msg) => write!(f, "Process launch error: {msg}"),
            NexusError::ProcessStop(msg) => write!(f, "Process stop error: {msg}"),
            NexusError::FileOperation(msg) => write!(f, "File operation error: {msg}"),
            NexusError::ResourceLoading(msg) => write!(f, "Resource loading error: {msg}"),
        }
    }
}

impl std::error::Error for NexusError {}

/// Type alias for Results using NexusError
pub type Result<T> = std::result::Result<T, NexusError>;
