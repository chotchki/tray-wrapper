/// The state of the running server process
#[derive(Debug, Default)]
pub enum ServerStatus {
    #[default]
    StartUp,
    Running(String),
    Stopped(String),
    Error(String),
}
