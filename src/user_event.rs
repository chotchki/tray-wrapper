use crate::server_status::ServerStatus;

#[derive(Debug)]
pub enum UserEvent {
    TrayIcon(tray_icon::TrayIconEvent),
    Menu(tray_icon::menu::MenuEvent),
    ServerStatus(ServerStatus),
    ServerExit,
}
