use crate::server_generator::{ContinueRunning, ServerGenerator};
use crate::server_status::ServerStatus;

/// Returned when the receiving end has gone away. The loop treats this as a
/// signal that there's no point in continuing — the wrapper is shutting down.
#[derive(Debug)]
pub(crate) struct SendError;

/// Abstraction over "where do status updates and exit notifications go."
/// In production this is `EventLoopProxy<UserEvent>`; in tests it's a
/// `Vec`-backed mock so we can assert the exact event sequence.
pub(crate) trait StatusSender: Send {
    fn send_status(&self, status: ServerStatus) -> Result<(), SendError>;
    fn send_exit(&self) -> Result<(), SendError>;
}

/// Runs the server generator in a loop, emitting status updates between
/// iterations and exit signals on terminal outcomes. Returns when the
/// generator yields `Exit` / `ExitWithError`, or when the sender drops.
pub(crate) async fn run_server_loop<S: StatusSender>(sg: ServerGenerator, sender: S) {
    loop {
        let next_run = sg();
        if sender.send_status(ServerStatus::Running).is_err() {
            break;
        }
        match next_run.await {
            ContinueRunning::Continue => {
                if sender
                    .send_status(ServerStatus::Stopped(
                        "Server Exited, will start again".to_string(),
                    ))
                    .is_err()
                {
                    break;
                }
                continue;
            }
            ContinueRunning::Exit => {
                let _ = sender.send_exit();
                break;
            }
            ContinueRunning::ExitWithError(e) => {
                let _ = sender.send_status(ServerStatus::Error(e));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, PartialEq, Eq)]
    enum MockEvent {
        Status(ServerStatus),
        Exit,
    }

    #[derive(Clone, Default)]
    struct MockSender {
        events: Arc<Mutex<Vec<MockEvent>>>,
    }

    impl MockSender {
        fn drain(&self) -> Vec<MockEvent> {
            self.events.lock().unwrap().drain(..).collect()
        }
    }

    impl StatusSender for MockSender {
        fn send_status(&self, status: ServerStatus) -> Result<(), SendError> {
            self.events.lock().unwrap().push(MockEvent::Status(status));
            Ok(())
        }
        fn send_exit(&self) -> Result<(), SendError> {
            self.events.lock().unwrap().push(MockEvent::Exit);
            Ok(())
        }
    }

    fn block_on<F: Future<Output = ()>>(f: F) {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f);
    }

    fn stopped_msg() -> ServerStatus {
        ServerStatus::Stopped("Server Exited, will start again".to_string())
    }

    #[test]
    fn immediate_exit_emits_running_then_exit() {
        let sender = MockSender::default();
        let sg: ServerGenerator = Arc::new(|| Box::pin(async { ContinueRunning::Exit }));

        block_on(run_server_loop(sg, sender.clone()));

        assert_eq!(
            sender.drain(),
            vec![MockEvent::Status(ServerStatus::Running), MockEvent::Exit]
        );
    }

    #[test]
    fn immediate_exit_with_error_emits_running_then_error_no_exit() {
        let sender = MockSender::default();
        let sg: ServerGenerator =
            Arc::new(|| Box::pin(async { ContinueRunning::ExitWithError("boom".to_string()) }));

        block_on(run_server_loop(sg, sender.clone()));

        // No MockEvent::Exit — by design ExitWithError keeps the event loop
        // alive so the user can read the error before quitting manually.
        assert_eq!(
            sender.drain(),
            vec![
                MockEvent::Status(ServerStatus::Running),
                MockEvent::Status(ServerStatus::Error("boom".to_string())),
            ]
        );
    }

    #[test]
    fn continue_then_exit_runs_generator_twice() {
        let sender = MockSender::default();
        let count = Arc::new(Mutex::new(0u32));
        let sg: ServerGenerator = {
            let count = count.clone();
            Arc::new(move || {
                let count = count.clone();
                Box::pin(async move {
                    let mut c = count.lock().unwrap();
                    *c += 1;
                    if *c == 1 {
                        ContinueRunning::Continue
                    } else {
                        ContinueRunning::Exit
                    }
                })
            })
        };

        block_on(run_server_loop(sg, sender.clone()));

        assert_eq!(*count.lock().unwrap(), 2);
        assert_eq!(
            sender.drain(),
            vec![
                MockEvent::Status(ServerStatus::Running),
                MockEvent::Status(stopped_msg()),
                MockEvent::Status(ServerStatus::Running),
                MockEvent::Exit,
            ]
        );
    }

    #[test]
    fn continue_then_exit_with_error_runs_generator_twice() {
        let sender = MockSender::default();
        let count = Arc::new(Mutex::new(0u32));
        let sg: ServerGenerator = {
            let count = count.clone();
            Arc::new(move || {
                let count = count.clone();
                Box::pin(async move {
                    let mut c = count.lock().unwrap();
                    *c += 1;
                    if *c == 1 {
                        ContinueRunning::Continue
                    } else {
                        ContinueRunning::ExitWithError("boom".to_string())
                    }
                })
            })
        };

        block_on(run_server_loop(sg, sender.clone()));

        assert_eq!(*count.lock().unwrap(), 2);
        assert_eq!(
            sender.drain(),
            vec![
                MockEvent::Status(ServerStatus::Running),
                MockEvent::Status(stopped_msg()),
                MockEvent::Status(ServerStatus::Running),
                MockEvent::Status(ServerStatus::Error("boom".to_string())),
            ]
        );
    }
}
