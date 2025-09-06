use std::{pin::Pin, sync::Arc};

/// This is the core of the library, in short you need to supply a function that
/// when called returns async futures that can be polled.
pub type ServerGenerator = Arc<dyn Fn() -> ServerGeneratorResult + Send + Sync>;

/// This is the return type for the async tasks, the future's return type is to indicate
/// to the wrapper if it should keep being executed.
pub type ServerGeneratorResult = Pin<Box<dyn Future<Output = ContinueRunning> + Send>>;

/// The various options on if it should continue to execute the server again.
pub enum ContinueRunning {
    /// The server generator should be called again
    Continue,
    /// There is no point in trying to execute again
    Exit,
    /// There is no point in trying to execute again and we have a reason
    ExitWithError(String),
}
