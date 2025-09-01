use std::sync::Arc;
use tray_wrapper::{ContinueRunning, ServerGeneratorResult, TrayWrapper, setup_event_loop};

fn main() -> anyhow::Result<()> {
    let event_loop = setup_event_loop();
    fn sg() -> ServerGeneratorResult {
        let task = async { ContinueRunning::Exit };
        Box::pin(task)
    }

    let mut tw = TrayWrapper::new(
        include_bytes!("../examples/example_icon.png"),
        event_loop.create_proxy(),
        Arc::new(&sg),
    )
    .unwrap();

    //Fix to ensure GTK has been started on linux (see tray-icon examples)
    #[cfg(target_os = "linux")]
    {
        gtk::init().unwrap();
    }

    if let Err(err) = event_loop.run_app(&mut tw) {
        println!("Error: {err:?}");
    }

    Ok(())
}
