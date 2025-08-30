use crate::server_status::ServerStatus;

#[derive(Debug)]
pub enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
    ServerStatusEvent(ServerStatus),
    ServerExitEvent,
}
