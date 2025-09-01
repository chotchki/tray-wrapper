//! The tray wrapper library is intended to make it simple to provide a GUI tray icon for a given running server
//! process. This uses the excellent tray-icon library for the UI and re-exports some of its types.
//!
//! This is to make it convient to start and monitor a process on Mac (other operating systems are supported
//! on a best effort basis). MacOS makes it challenging to manage a server process WITHOUT a tray icon.
//!
//! The tray icon provides a submenu to view the supplied server status and the ability to exit.
mod event_loop;
mod menu_state;
mod server_generator;
mod server_status;
mod tray_wrapper;
mod user_event;

//Public interface
pub use event_loop::setup_event_loop;
pub use server_generator::{ContinueRunning, ServerGeneratorResult};
pub use tray_wrapper::{TrayWrapper, TrayWrapperError};
