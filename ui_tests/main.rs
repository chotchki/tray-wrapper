use std::{pin::Pin, sync::Arc};

use anyhow::Error;
use tokio::sync::mpsc::{Sender, channel};
use tray_wrapper::{TrayWrapper, event_loop::setup_event_loop, user_event::UserEvent};
use winit::event_loop::EventLoopProxy;

fn main() -> anyhow::Result<()> {
    let event_loop = setup_event_loop();
    fn sg(
        _: EventLoopProxy<UserEvent>,
    ) -> Result<Pin<Box<dyn Future<Output = ()> + Send + 'static>>, Box<dyn std::error::Error>>
    {
        let task = async {};
        Ok(Box::pin(task))
    }
    let tw = TrayWrapper::new(
        include_bytes!("../examples/example_icon.png"),
        Arc::new(&sg),
        event_loop.create_proxy(),
    )
    .unwrap();
    Ok(())
}
