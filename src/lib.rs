//! The tray wrapper library is intended to make it simple to provide a GUI tray icon for a given running server
//! process. This uses the excellent tray-icon library for the UI and re-exports some of its types.
//!
//! This is to make it convient to start and monitor a process on Mac (other operating systems are supported
//! on a best effort basis). MacOS makes it challenging to manage a server process WITHOUT a tray icon.
//!
//! The tray icon provides a submenu to view the supplied server status and the ability to exit.

use std::pin::Pin;

use image::ImageError;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use winit::window::{BadIcon, Icon};

/// The state of the running server process
#[derive(Debug, Default)]
pub enum ServerStatus {
    #[default]
    StartUp,
    Running(String),
    Stopped(String),
    Error(String),
}

/// The ServerGenerator is a closure that will be called time to generate the server to be run.
///
/// This is where you would read any configuration files or do other setup to be ready for it to be
/// run.
pub type ServerGenerator<E> = Box<
    dyn Fn(
        CancellationToken,
        Receiver<ServerStatus>,
    ) -> Result<Pin<Box<dyn Future<Output = ()>>>, E>,
>;

/// This is the main entry point / handle for the wrapper
pub struct TrayWrapper<E> {
    icon: Icon,
    server_generator: ServerGenerator<E>,
}

impl<E> TrayWrapper<E> {
    ///Construct the wrapper, its recommended you compile time load the icon which means you
    /// can ignore image parsing errors.
    pub fn new(
        icon_data: &[u8],
        server_generator: ServerGenerator<E>,
    ) -> Result<Self, TrayWrapperError> {
        let image = image::load_from_memory(icon_data)?.into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height)?;

        Ok(TrayWrapper {
            icon,
            server_generator,
        })
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

    use std::sync::mpsc::Sender;

    use tokio::sync::mpsc::channel;

    use super::*;

    #[test]
    fn example() -> anyhow::Result<()> {
        fn sg(
            _: CancellationToken,
            _: Receiver<ServerStatus>,
        ) -> Result<Pin<Box<dyn Future<Output = ()>>>, anyhow::Error> {
            let task = async {};
            Ok(Box::pin(task))
        }
        let c = CancellationToken::new();
        let (send, recv) = channel::<(Sender<ServerStatus>, Receiver<ServerStatus>)>(1);
        let tw = TrayWrapper::new(
            include_bytes!("../examples/example_icon.png"),
            Box::new(&sg),
        )?;
        Ok(())
    }
}
