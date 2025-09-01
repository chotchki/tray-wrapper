use crate::{
    event_loop::UserEvent,
    menu_state::MenuState,
    server_generator::{ContinueRunning, ServerGenerator},
    server_status::ServerStatus,
};
use image::ImageError;
use std::time::Duration;
use take_once::TakeOnce;
use thiserror::Error;
use tokio::runtime::Runtime;
use tray_icon::{BadIcon, Icon};
use winit::{application::ApplicationHandler, event_loop::EventLoopProxy};

/// This is the main entry point / handle for the wrapper
pub struct TrayWrapper {
    icon: Icon,
    menu_state: Option<MenuState>,
    runtime: Option<Runtime>,
    event_loop_proxy: EventLoopProxy<UserEvent>,
    server_generator: TakeOnce<ServerGenerator>,
}

impl TrayWrapper {
    ///Construct the wrapper, its recommended you compile time load the icon which means you
    /// can ignore image parsing errors.
    pub fn new(
        icon_data: &[u8],
        event_loop_proxy: EventLoopProxy<UserEvent>,
        server_gen: ServerGenerator,
    ) -> Result<Self, TrayWrapperError> {
        let image = image::load_from_memory(icon_data)?.into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height)?;
        let server_generator = TakeOnce::new_with(server_gen);

        Ok(TrayWrapper {
            icon,
            menu_state: None,
            runtime: Some(Runtime::new()?),

            event_loop_proxy,
            server_generator,
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

            let sg = self
                .server_generator
                .take()
                .expect("Unable to take generator function");
            let elp = self.event_loop_proxy.clone();
            rt.spawn(async move {
                let sg_fn = sg;
                loop {
                    let next_run = sg_fn();
                    elp.send_event(UserEvent::ServerStatus(ServerStatus::Running))
                        .expect("Event Loop Closed!");
                    match next_run.await {
                        ContinueRunning::Continue => {
                            elp.send_event(UserEvent::ServerStatus(ServerStatus::Stopped(
                                "Server Exited, will start again".to_string(),
                            )))
                            .expect("Event Loop Closed!");
                            continue;
                        }
                        ContinueRunning::Exit => {
                            elp.send_event(UserEvent::ServerExit)
                                .expect("Event Loop Closed!");
                            break;
                        }
                        ContinueRunning::ExitWithError(e) => {
                            elp.send_event(UserEvent::ServerStatus(ServerStatus::Error(
                                e.to_string(),
                            )))
                            .expect("Event Loop Closed!");
                            break;
                        }
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
        if let UserEvent::ServerExit = event {
            if let Some(rt) = self.runtime.take() {
                rt.shutdown_timeout(Duration::from_secs(10));
            }
            _event_loop.exit();
        }

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
