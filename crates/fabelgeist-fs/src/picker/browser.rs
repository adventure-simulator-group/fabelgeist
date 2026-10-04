use super::{BrowserPickerStage, FilePickerFilter, PickerError, PickerKind};
use crate::{DirectoryEntry, FileEntry, WebDirectory, WebFile};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub(super) struct BrowserPicker(js_sys::Object);
impl BrowserPicker {
    pub(super) fn window(kind: PickerKind) -> Result<Self, PickerError> {
        match web_sys::window() {
            Some(window) => Ok(Self(window.into())),
            None => Err(PickerError::MissingWindow { kind }),
        }
    }

    pub(super) async fn file(
        &self,
        filter: &FilePickerFilter,
    ) -> Result<Option<Box<dyn FileEntry>>, PickerError> {
        let kind = PickerKind::File;
        let result = JsFuture::from(self.invoke(PickerRequest::File(filter))?)
            .await
            .map_err(|source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::AwaitPicker, source)
            })?;
        let handles =
            result
                .dyn_into::<js_sys::Array>()
                .map_err(|source: JsValue| -> PickerError {
                    PickerError::from_browser(kind, BrowserPickerStage::DecodeArray, source)
                })?;
        // Reflect catches property exceptions even on an array proxy.
        let length = js_sys::Reflect::get(&handles, &"length".into()).map_err(
            |source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::ReadArrayLength, source)
            },
        )?;
        match length.as_f64() {
            Some(0.0) => return Ok(None),
            Some(1.0) => {}
            Some(count) if count.is_finite() && count > 1.0 && count.fract() == 0.0 => {
                return Err(PickerError::from_browser(
                    kind,
                    BrowserPickerStage::FileCardinality,
                    handles.into(),
                ));
            }
            _ => {
                return Err(PickerError::from_browser(
                    kind,
                    BrowserPickerStage::DecodeArrayLength,
                    length,
                ));
            }
        }
        let value = js_sys::Reflect::get(&handles, &0.into()).map_err(
            |source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::ReadFileHandle, source)
            },
        )?;
        let handle = value.dyn_into::<web_sys::FileSystemFileHandle>().map_err(
            |source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::DecodeFileHandle, source)
            },
        )?;
        Ok(Some(Box::new(WebFile::from(handle))))
    }

    pub(super) async fn directory(&self) -> Result<Option<Box<dyn DirectoryEntry>>, PickerError> {
        let kind = PickerKind::Directory;
        let result = JsFuture::from(self.invoke(PickerRequest::Directory)?)
            .await
            .map_err(|source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::AwaitPicker, source)
            })?;
        let handle = result
            .dyn_into::<web_sys::FileSystemDirectoryHandle>()
            .map_err(|source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::DecodeDirectoryHandle, source)
            })?;
        Ok(Some(Box::new(WebDirectory::from(handle))))
    }

    fn invoke(&self, request: PickerRequest<'_>) -> Result<js_sys::Promise, PickerError> {
        let kind = request.kind();
        let options = match request {
            PickerRequest::File(filter) => Some(filter.browser_options()?),
            PickerRequest::Directory => None,
        };
        let method = match request {
            PickerRequest::File(_) => "showOpenFilePicker",
            PickerRequest::Directory => "showDirectoryPicker",
        };
        let method = js_sys::Reflect::get(&self.0, &method.into()).map_err(
            |source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::ReadMethod, source)
            },
        )?;
        let method =
            method
                .dyn_into::<js_sys::Function>()
                .map_err(|source: JsValue| -> PickerError {
                    PickerError::from_browser(kind, BrowserPickerStage::DecodeMethod, source)
                })?;
        let result = match options {
            Some(options) => method.call1(&self.0, &options),
            None => method.call0(&self.0),
        }
        .map_err(|source: JsValue| -> PickerError {
            PickerError::from_browser(kind, BrowserPickerStage::InvokeMethod, source)
        })?;
        result
            .dyn_into::<js_sys::Promise>()
            .map_err(|source: JsValue| -> PickerError {
                PickerError::from_browser(kind, BrowserPickerStage::DecodePromise, source)
            })
    }
}

#[derive(Clone, Copy)]
enum PickerRequest<'a> {
    File(&'a FilePickerFilter),
    Directory,
}
impl PickerRequest<'_> {
    fn kind(self) -> PickerKind {
        match self {
            Self::File(_) => PickerKind::File,
            Self::Directory => PickerKind::Directory,
        }
    }
}
impl PickerError {
    pub(super) fn from_browser(
        kind: PickerKind,
        stage: BrowserPickerStage,
        source: JsValue,
    ) -> Self {
        Self::Browser {
            kind,
            stage,
            source: crate::BrowserIoCause::from(source),
        }
    }
}

#[cfg(test)]
mod tests;
