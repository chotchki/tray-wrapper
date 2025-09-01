use std::sync::{Arc, LazyLock, Mutex};
use tray_wrapper::{ContinueRunning, ServerGeneratorResult, create_tray_wrapper};

static RUNS_COUNT: LazyLock<Mutex<u8>> = LazyLock::new(|| Mutex::new(0));

//This test should succeed, restart and then exit, it has to be run this way due to main thread requirements.
fn main() -> anyhow::Result<()> {
    fn sg() -> ServerGeneratorResult {
        let task = async {
            let mut runs = RUNS_COUNT.lock().unwrap();
            *runs += 1;

            if *runs == 1 {
                ContinueRunning::Exit
            } else {
                ContinueRunning::Continue
            }
        };
        Box::pin(task)
    }

    create_tray_wrapper(
        include_bytes!("../examples/example_icon.png"),
        Arc::new(&sg),
    )?;

    let runs = RUNS_COUNT.lock().unwrap();
    assert_eq!(*runs, 1);

    Ok(())
}
