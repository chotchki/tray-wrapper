//! This the types needed to conform to in order to
//! have the wrapper execute your server.

use std::{pin::Pin, sync::Arc};

pub enum ContinueRunning {
    Continue,
    Exit,
    ExitWithError(String),
}

pub type ServerGeneratorResult = Pin<Box<dyn Future<Output = ContinueRunning> + Send>>;

pub type ServerGenerator = Arc<dyn Fn() -> ServerGeneratorResult + Send + Sync>;
