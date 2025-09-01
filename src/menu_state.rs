use thiserror::Error;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

use crate::{server_status::ServerStatus, user_event::UserEvent};

pub struct MenuState {
    tray_icon: TrayIcon,
    status_item: MenuItem,
    quit_item: MenuItem,
    tray_menu: Menu,
}

impl MenuState {
    pub fn new(icon: Icon) -> Result<Self, MenuStateError> {
        let tray_icon = TrayIconBuilder::new().with_icon(icon).build()?;

        let status_item = MenuItem::new("Starting Up", false, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let tray_menu = Menu::new();
        tray_menu.append(&status_item)?;
        tray_menu.append(&quit_item)?;

        Ok(Self {
            tray_icon,
            status_item,
            quit_item,
            tray_menu,
        })
    }

    pub fn update_tray_icon(&mut self, status: ServerStatus) {
        match status {
            ServerStatus::StartUp => {
                self.tray_icon.set_title(Some("?"));
                self.status_item.set_text("In startup");
            }
            ServerStatus::Running => {
                self.tray_icon.set_title(None as Option<String>);
                self.status_item.set_text("Running");
            }
            ServerStatus::Stopped(s) => {
                self.tray_icon.set_title(Some("X"));
                self.status_item.set_text(s);
            }
            ServerStatus::Error(e) => {
                self.tray_icon.set_title(Some("E"));
                self.status_item.set_text(e);
            }
        }
        self.tray_icon
            .set_menu(Some(Box::new(self.tray_menu.clone())));
    }

    pub fn quit_matches(&self, event: UserEvent) -> bool {
        if let UserEvent::MenuEvent(me) = event
            && me.id == self.quit_item.id()
        {
            true
        } else {
            false
        }
    }
}

#[derive(Error, Debug)]
pub enum MenuStateError {
    #[error(transparent)]
    MenuError(#[from] tray_icon::menu::Error),
    #[error(transparent)]
    TrayError(#[from] tray_icon::Error),
}
