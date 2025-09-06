//! Running the Axum Example
//! https://github.com/tokio-rs/axum/tree/v0.8.x

use axum::{Router, routing::get};
use std::sync::Arc;
use tray_wrapper::{ContinueRunning, ServerGeneratorResult, create_tray_wrapper};

async fn root() -> &'static str {
    "Hello, World!"
}

fn main() -> anyhow::Result<()> {
    fn sg() -> ServerGeneratorResult {
        let task = async {
            // build our application with a route
            let app = Router::new()
                // `GET /` goes to `root`
                .route("/", get(root));

            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
            ContinueRunning::Continue
        };
        Box::pin(task)
    }

    create_tray_wrapper(
        include_bytes!("../examples/example_icon.png"),
        None,
        Arc::new(&sg),
    )?;

    Ok(())
}
