
/// Enum to store the error occurred during a call to a render function.
#[derive(Debug, Clone)]
pub enum Error {
    /// Engine could not be initialized error.
    /// The exact (OS) error message is also reported as an argument
    InitializeError(String),

    /// Error reported during a window creation.
    WindowCreationError(String),
    /// Error reported by the OS during rendering
    ContextError(String),
    /// Error reported due to missing or closed context. For example window has been closed.
    ContextLost,

    /// Error occured during a render pass creation
    PassCreationError(String),
}