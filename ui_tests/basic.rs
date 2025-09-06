use std::sync::Arc;
use tray_wrapper::{ContinueRunning, ServerGeneratorResult, create_tray_wrapper};

fn main() -> anyhow::Result<()> {
    fn sg() -> ServerGeneratorResult {
        let task = async { ContinueRunning::Exit };
        Box::pin(task)
    }

    create_tray_wrapper(
        include_bytes!("../examples/example_icon.png"),
        None,
        Arc::new(&sg),
    )?;

    Ok(())
}
