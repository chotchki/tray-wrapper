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
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;
use tray_icon::{
    BadIcon, Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};
use winit::application::ApplicationHandler;

use crate::menu_state::MenuState;
mod menu_state;

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
    dyn Fn(CancellationToken, Sender<ServerStatus>) -> Result<Pin<Box<dyn Future<Output = ()>>>, E>,
>;

/// This is the main entry point / handle for the wrapper
pub struct TrayWrapper<E> {
    icon: Icon,
    menu_state: Option<MenuState>,

    server_generator: ServerGenerator<E>,
    cancel_token: CancellationToken,
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
            menu_state: None,

            server_generator,
            cancel_token: CancellationToken::new(),
        })
    }
}

#[derive(Debug)]
pub enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

// This implementation is from the winit example here: https://github.com/tauri-apps/tray-icon/blob/dev/examples/winit.rs
impl<E> ApplicationHandler<UserEvent> for TrayWrapper<E> {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        // We create the icon once the event loop is actually running
        // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
        if winit::event::StartCause::Init == cause {
            let Ok(mut ms) = MenuState::new(self.icon.clone()) else {
                return _event_loop.exit();
            };
            ms.update_tray_icon(ServerStatus::StartUp);
            self.menu_state = Some(ms);

            //Now its time to really start the server
        }

        // We have to request a redraw here to have the icon actually show up.
        // Winit only exposes a redraw method on the Window so we use core-foundation directly-ish.
        #[cfg(target_os = "macos")]
        {
            use objc2_core_foundation::CFRunLoop;
            let rl = CFRunLoop::main().unwrap();
            CFRunLoop::wake_up(&rl);
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        if let Some(ms) = &self.menu_state
            && ms.quit_matches(event)
        {
            _event_loop.exit();
        }
    }
}

#[derive(Error, Debug)]
pub enum TrayWrapperError {
    #[error("Unable to load the icon from buffer")]
    IconLoad(#[from] ImageError),
    #[error("Tray Icon Bad Icon")]
    BadIcon(#[from] BadIcon),
    #[error("Failure to pre-create menu")]
    MenuError(#[from] tray_icon::menu::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::{Sender, channel};

    #[test]
    fn example() -> anyhow::Result<()> {
        fn sg(
            _: CancellationToken,
            _: Sender<ServerStatus>,
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
