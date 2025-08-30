/// The state of the running server process
#[derive(Debug, Default)]
pub enum ServerStatus {
    //This is used when the wrapper starts up
    #[default]
    StartUp,

    //This is set just before await is called on your server
    Running,

    //This is set if the server stops but indicates it can be resumed, this may not be needed
    Stopped(String),

    //This is for when the server fails
    Error(String),
}
