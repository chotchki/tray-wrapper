//! The tray wrapper library is intended to make it simple to provide a GUI tray icon for a given running server
//! process. This uses the excellent tray-icon library for the UI and re-exports some of its types.
//!
//! This is to make it convient to start and monitor a process on Mac (other operating systems are supported
//! on a best effort basis). MacOS makes it challenging to manage a server process WITHOUT a tray icon.
//!
//! The tray icon provides a submenu to view the supplied server status and the ability to exit.

use image::ImageError;
use thiserror::Error;
use winit::window::{BadIcon, Icon};

/// The state of the running server process
#[derive(Debug, Default)]
enum ServerStatus {
    #[default]
    StartUp,
    Running(String),
    Stopped(String),
    Error(String),
}

/// This is the main entry point / handle for the wrapper
pub struct TrayWrapper {
    icon: Icon,
}

impl TrayWrapper {
    ///Construct the wrapper
    ///
    /// ```
    /// # use tray_wrapper::{TrayWrapper, TrayWrapperError};
    /// let tw = TrayWrapper::new(include_bytes!("../examples/example_icon.png"))?;
    /// # Ok::<(), TrayWrapperError>(())
    /// ```
    pub fn new(icon_data: &[u8]) -> Result<Self, TrayWrapperError> {
        let image = image::load_from_memory(icon_data)?.into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height)?;

        Ok(TrayWrapper { icon })
    }
}

#[derive(Error, Debug)]
pub enum TrayWrapperError {
    #[error("Unable to load the icon from buffer")]
    IconLoadError(#[from] ImageError),
    #[error("Winit Bad Icon")]
    BadIcon(#[from] BadIcon),
}

#[cfg(test)]
mod tests {
    //use super::*;
}
