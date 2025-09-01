use tray_icon::{TrayIconEvent, menu::MenuEvent};
use winit::event_loop::EventLoop;

use crate::user_event::UserEvent;

pub fn setup_event_loop() -> EventLoop<UserEvent> {
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(UserEvent::TrayIcon(event))
            .expect("Tray Icon Event loop doesn't exist");
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        proxy
            .send_event(UserEvent::Menu(event))
            .expect("Menu Event loop doesn't exist");
    }));

    event_loop
}
