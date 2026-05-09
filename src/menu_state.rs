use thiserror::Error;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

use crate::{event_loop::UserEvent, server_status::ServerStatus};

pub struct MenuState {
    tray_icon: TrayIcon,
    status_item: MenuItem,
    quit_item: MenuItem,
    tray_menu: Menu,
}

impl MenuState {
    pub fn new(icon: Icon, version: Option<String>) -> Result<Self, MenuStateError> {
        let tray_icon = TrayIconBuilder::new().with_icon(icon).build()?;

        let status_item = MenuItem::new("Starting Up", false, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let tray_menu = Menu::new();
        tray_menu.append(&status_item)?;
        if let Some(v) = version {
            tray_menu.append(&MenuItem::new(v, false, None))?;
        }
        tray_menu.append(&quit_item)?;

        Ok(Self {
            tray_icon,
            status_item,
            quit_item,
            tray_menu,
        })
    }

    pub fn update_tray_icon(&mut self, status: ServerStatus) {
        let (title, text) = status_display(&status);
        self.tray_icon.set_title(Some(title));
        self.status_item.set_text(text);
        self.tray_icon
            .set_menu(Some(Box::new(self.tray_menu.clone())));
    }

    pub fn quit_matches(&self, event: &UserEvent) -> bool {
        matches!(event, UserEvent::Menu(me) if me.id == self.quit_item.id())
    }
}

fn status_display(status: &ServerStatus) -> (&'static str, &str) {
    match status {
        ServerStatus::StartUp => ("?", "In startup"),
        ServerStatus::Running => ("", "Running"),
        ServerStatus::Stopped(s) => ("X", s),
        ServerStatus::Error(e) => ("E", e),
    }
}

#[derive(Error, Debug)]
pub enum MenuStateError {
    #[error(transparent)]
    MenuError(#[from] tray_icon::menu::Error),
    #[error(transparent)]
    TrayError(#[from] tray_icon::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_display() {
        assert_eq!(status_display(&ServerStatus::StartUp), ("?", "In startup"));
    }

    #[test]
    fn running_display() {
        assert_eq!(status_display(&ServerStatus::Running), ("", "Running"));
    }

    #[test]
    fn stopped_display_passes_through_message() {
        let status = ServerStatus::Stopped("custom stop message".to_string());
        assert_eq!(status_display(&status), ("X", "custom stop message"));
    }

    #[test]
    fn error_display_passes_through_message() {
        let status = ServerStatus::Error("something failed".to_string());
        assert_eq!(status_display(&status), ("E", "something failed"));
    }
}
