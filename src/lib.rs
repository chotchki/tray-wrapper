//! The tray wrapper library is intended to make it simple to provide a GUI tray icon for a given running server
//! process. This uses the excellent tray-icon library for the UI and re-exports some of its types.
//!
//! This is to make it convient to start and monitor a process on Mac (other operating systems are supported
//! on a best effort basis). MacOS makes it challenging to manage a server process WITHOUT a tray icon.
//!
//! The tray icon provides a submenu to view the supplied server status and the ability to exit.
pub mod event_loop;
mod menu_state;
mod server_status;
pub mod user_event;

use std::{pin::Pin, sync::Arc, time::Duration};

use image::ImageError;
use thiserror::Error;
use tokio::runtime::Runtime;
use tray_icon::{BadIcon, Icon};
use winit::{application::ApplicationHandler, event_loop::EventLoopProxy};

use crate::{
    menu_state::{MenuState, MenuStateError},
    server_status::ServerStatus,
    user_event::UserEvent,
};

/// The ServerGenerator is a closure that will be called repeated to generate the server to be run.
///
/// This is where you would read any configuration files or do other setup to be ready for it to be
/// run.
pub type ServerGenerator = Arc<
    dyn Fn(
            EventLoopProxy<UserEvent>,
        )
            -> Result<Pin<Box<dyn Future<Output = ()> + Send + 'static>>, Box<dyn std::error::Error>>
        + Send
        + Sync,
>;

/// This is the main entry point / handle for the wrapper
pub struct TrayWrapper {
    icon: Icon,
    menu_state: Option<MenuState>,
    runtime: Option<Runtime>,

    server_generator: ServerGenerator,
    event_loop_proxy: EventLoopProxy<UserEvent>,
}

impl TrayWrapper {
    ///Construct the wrapper, its recommended you compile time load the icon which means you
    /// can ignore image parsing errors.
    pub fn new(
        icon_data: &[u8],
        server_generator: ServerGenerator,
        event_loop_proxy: EventLoopProxy<UserEvent>,
    ) -> Result<Self, TrayWrapperError> {
        let image = image::load_from_memory(icon_data)?.into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height)?;

        Ok(TrayWrapper {
            icon,
            menu_state: None,
            runtime: Some(Runtime::new()?),

            server_generator,
            event_loop_proxy,
        })
    }
}

// This implementation is from the winit example here: https://github.com/tauri-apps/tray-icon/blob/dev/examples/winit.rs
impl ApplicationHandler<UserEvent> for TrayWrapper {
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
            ms.update_tray_icon(ServerStatus::StartUp); //The error type doesn't matter in this case
            self.menu_state = Some(ms);

            //Now its time to really start the server
            let Some(rt) = &self.runtime else {
                return _event_loop.exit();
            };

            let sg = self.server_generator.clone();
            let elp = self.event_loop_proxy.clone();
            rt.spawn(async move {
                loop {
                    match sg(elp.clone()) {
                        Ok(_) => elp
                            .send_event(UserEvent::ServerStatusEvent(ServerStatus::Stopped(
                                "The server stopped, restarting".to_string(),
                            )))
                            .expect("Event Loop Closed!"),
                        Err(e) => elp
                            .send_event(UserEvent::ServerStatusEvent(ServerStatus::Error(
                                e.to_string(),
                            )))
                            .expect("Event Loop Closed!"),
                    }
                }
            });
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
            if let Some(rt) = self.runtime.take() {
                rt.shutdown_timeout(Duration::from_secs(10));
            }
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
    #[error(transparent)]
    RunTime(#[from] std::io::Error),
}
