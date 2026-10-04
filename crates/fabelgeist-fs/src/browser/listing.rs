use super::{BrowserIoCause, WebDirectory, WebFile};
use crate::{BrowserListingStage, DirectoryListingError, Entry};
use wasm_bindgen::{JsCast, JsValue};

pub(super) struct DirectoryListingIterator {
    directory: WebDirectory,
    iterator: js_sys::Object,
    next_method: js_sys::Function,
}

pub(super) enum ListingStep {
    Finished,
    Child(Entry),
}

impl DirectoryListingIterator {
    pub(super) fn new(directory: &WebDirectory) -> Result<Self, DirectoryListingError> {
        let method = js_sys::Reflect::get(&directory.handle, &"values".into()).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::ReadValuesMethod,
                    source,
                )
            },
        )?;
        let method = method.dyn_into::<js_sys::Function>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::DecodeValuesMethod,
                    source,
                )
            },
        )?;
        let iterator = method.call0(&directory.handle).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::InvokeValuesMethod,
                    source,
                )
            },
        )?;
        let iterator = iterator.dyn_into::<js_sys::Object>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::DecodeIterator,
                    source,
                )
            },
        )?;
        let next_method = js_sys::Reflect::get(&iterator, &"next".into()).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::ReadNextMethod,
                    source,
                )
            },
        )?;
        let next_method = next_method.dyn_into::<js_sys::Function>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::DecodeNextMethod,
                    source,
                )
            },
        )?;
        Ok(Self {
            directory: directory.clone(),
            iterator,
            next_method,
        })
    }

    pub(super) async fn next(&mut self) -> Result<ListingStep, DirectoryListingError> {
        let promise = self.next_method.call0(&self.iterator).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    &self.directory,
                    BrowserListingStage::InvokeNextMethod,
                    source,
                )
            },
        )?;
        let promise = promise.dyn_into::<js_sys::Promise>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    &self.directory,
                    BrowserListingStage::DecodeNextPromise,
                    source,
                )
            },
        )?;
        let result = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    &self.directory,
                    BrowserListingStage::AwaitNext,
                    source,
                )
            })?;
        ListingStep::from_browser(&self.directory, result)
    }
}

impl ListingStep {
    fn from_browser(
        directory: &WebDirectory,
        result: JsValue,
    ) -> Result<Self, DirectoryListingError> {
        let result = result.dyn_into::<js_sys::Object>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::DecodeResult,
                    source,
                )
            },
        )?;
        let done = js_sys::Reflect::get(&result, &"done".into()).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::ReadDone,
                    source,
                )
            },
        )?;
        // IteratorComplete applies JavaScript ToBoolean; absent `done` is false.
        if done.is_truthy() {
            return Ok(Self::Finished);
        }
        let handle = js_sys::Reflect::get(&result, &"value".into()).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::ReadHandle,
                    source,
                )
            },
        )?;
        Entry::from_browser(directory, handle).map(Self::Child)
    }
}

impl Entry {
    fn from_browser(
        directory: &WebDirectory,
        handle: JsValue,
    ) -> Result<Self, DirectoryListingError> {
        let handle = handle.dyn_into::<js_sys::Object>().map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::DecodeHandle,
                    source,
                )
            },
        )?;
        let kind = js_sys::Reflect::get(&handle, &"kind".into()).map_err(
            |source: JsValue| -> DirectoryListingError {
                DirectoryListingError::from_browser(
                    directory,
                    BrowserListingStage::ReadKind,
                    source,
                )
            },
        )?;
        match kind.as_string().as_deref() {
            Some("file") => {
                let handle = handle.dyn_into::<web_sys::FileSystemFileHandle>().map_err(
                    |source: js_sys::Object| -> DirectoryListingError {
                        DirectoryListingError::from_browser(
                            directory,
                            BrowserListingStage::DecodeFileHandle,
                            source.into(),
                        )
                    },
                )?;
                Ok(Self::File(Box::new(WebFile::from(handle))))
            }
            Some("directory") => {
                let handle = handle
                    .dyn_into::<web_sys::FileSystemDirectoryHandle>()
                    .map_err(|source: js_sys::Object| -> DirectoryListingError {
                        DirectoryListingError::from_browser(
                            directory,
                            BrowserListingStage::DecodeDirectoryHandle,
                            source.into(),
                        )
                    })?;
                Ok(Self::Directory(Box::new(WebDirectory::from(handle))))
            }
            _ => Err(DirectoryListingError::from_browser(
                directory,
                BrowserListingStage::DecodeKind,
                kind,
            )),
        }
    }
}

impl DirectoryListingError {
    fn from_browser(directory: &WebDirectory, stage: BrowserListingStage, source: JsValue) -> Self {
        Self::Browser {
            directory: directory.clone(),
            stage,
            source: BrowserIoCause::from(source),
        }
    }
}

#[cfg(test)]
mod tests;
