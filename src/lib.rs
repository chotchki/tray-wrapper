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
use event_loop::setup_event_loop;
use tray_wrapper::{TrayWrapper, TrayWrapperError};

//Public interface
pub use server_generator::{ContinueRunning, ServerGenerator, ServerGeneratorResult};
use thiserror::Error;

pub fn create_tray_wrapper(
    icon_data: &[u8],
    version: Option<String>,
    server_gen: ServerGenerator,
) -> Result<(), CreateTrayWrapperError> {
    let event_loop = setup_event_loop();
    let mut tw = TrayWrapper::new(icon_data, version, event_loop.create_proxy(), server_gen)?;

    //Fix to ensure GTK has been started on linux (see tray-icon examples)
    #[cfg(target_os = "linux")]
    {
        gtk::init().unwrap();
    }

    event_loop.run_app(&mut tw)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum CreateTrayWrapperError {
    #[error(transparent)]
    TrayWrapper(#[from] TrayWrapperError),
    #[error(transparent)]
    Winit(#[from] winit::error::EventLoopError),
    #[cfg(target_os = "linux")]
    #[error("Gtk Failed to Init {0}")]
    Gtk(#[from] glib::error::BoolError),
}
