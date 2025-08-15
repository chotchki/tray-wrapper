//! The tray wrapper library is intended to make it simple to provide a GUI tray icon for a given running server
//! process.
//!
//! This is to make it convient to start and monitor a process on Mac (other operating systems are supported
//! on a best effort basis). MacOS makes it challenging to manage a server process WITHOUT a tray icon.

/// The state of the running server process
#[derive(Debug, Default)]
pub enum ServerStatus {
    #[default]
    StartUp,
    Running(String),
    Stopped(String),
    Error(String),
}

#[cfg(test)]
mod tests {
    //use super::*;
}
