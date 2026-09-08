use super::*;
use std::sync::mpsc::{self, Receiver};

pub(super) struct Backend {
    result: Option<Receiver<Result<BakedRecipe, String>>>,
}
impl Backend {
    pub fn new() -> Self {
        Self { result: None }
    }
    pub fn start(&mut self, document: Document) {
        let (sender, receiver) = mpsc::channel();
        self.result = Some(receiver);
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                BakedRecipe::generate(document.recipe, &document.texture)
            })
            .map_err(|_| "Recipe rejected the selected parameter combination".to_owned());
            let _ = sender.send(result);
        });
    }
    pub fn poll(&mut self) -> Option<Result<BakedRecipe, String>> {
        let result = self.result.as_ref()?.try_recv().ok()?;
        self.result = None;
        Some(result)
    }
    pub fn cancel(&mut self) -> bool {
        false
    }
}
