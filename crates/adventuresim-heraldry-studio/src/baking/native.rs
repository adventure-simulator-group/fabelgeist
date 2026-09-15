use std::sync::mpsc::{self, Receiver};
pub(super) struct Backend {
    result: Option<Receiver<Result<Vec<u8>, String>>>,
}
impl Backend {
    pub fn new() -> Self {
        Self { result: None }
    }
    pub fn start(&mut self, source: String) {
        let (sender, receiver) = mpsc::channel();
        self.result = Some(receiver);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| super::execute(&source))
                .unwrap_or_else(|_| Err("Worker panicked".into()));
            let _ = sender.send(result);
        });
    }
    pub fn poll(&mut self) -> Option<Result<Vec<u8>, String>> {
        let result = self.result.as_ref()?.try_recv().ok()?;
        self.result = None;
        Some(result)
    }
    pub fn cancel(&mut self) -> bool {
        false
    }
}
