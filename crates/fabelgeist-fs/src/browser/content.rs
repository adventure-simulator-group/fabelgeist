use super::WebFile;
use crate::{BrowserContentMethod, BrowserContentStage, FileContentError, FileContents, FileEntry};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[async_trait::async_trait(?Send)]
impl FileEntry for WebFile {
    fn name(&self) -> crate::EntryLabel {
        crate::EntryLabel::from(self.handle.name())
    }
    async fn read(&self) -> Result<FileContents, FileContentError> {
        let promise = self.invoke_content(&self.handle, ContentRequest::File)?;
        let value =
            JsFuture::from(promise)
                .await
                .map_err(|source: JsValue| -> FileContentError {
                    FileContentError::from_browser(
                        self,
                        BrowserContentMethod::File,
                        BrowserContentStage::AwaitFile,
                        source,
                    )
                })?;
        let file =
            value
                .dyn_into::<web_sys::File>()
                .map_err(|source: JsValue| -> FileContentError {
                    FileContentError::from_browser(
                        self,
                        BrowserContentMethod::File,
                        BrowserContentStage::DecodeFile,
                        source,
                    )
                })?;
        let promise = self.invoke_content(&file, ContentRequest::Buffer)?;
        let value =
            JsFuture::from(promise)
                .await
                .map_err(|source: JsValue| -> FileContentError {
                    FileContentError::from_browser(
                        self,
                        BrowserContentMethod::Buffer,
                        BrowserContentStage::AwaitBuffer,
                        source,
                    )
                })?;
        let buffer = value.dyn_into::<js_sys::ArrayBuffer>().map_err(
            |source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    BrowserContentMethod::Buffer,
                    BrowserContentStage::DecodeBuffer,
                    source,
                )
            },
        )?;
        Ok(FileContents::from(
            js_sys::Uint8Array::new(&buffer).to_vec(),
        ))
    }
    async fn write(&self, contents: &FileContents) -> Result<(), FileContentError> {
        let promise = self.invoke_content(&self.handle, ContentRequest::Writable)?;
        let value =
            JsFuture::from(promise)
                .await
                .map_err(|source: JsValue| -> FileContentError {
                    FileContentError::from_browser(
                        self,
                        BrowserContentMethod::Writable,
                        BrowserContentStage::AwaitWritable,
                        source,
                    )
                })?;
        let stream =
            value
                .dyn_into::<js_sys::Object>()
                .map_err(|source: JsValue| -> FileContentError {
                    FileContentError::from_browser(
                        self,
                        BrowserContentMethod::Writable,
                        BrowserContentStage::DecodeWritable,
                        source,
                    )
                })?;
        let promise = self.invoke_content(&stream, ContentRequest::Write(contents))?;
        JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    BrowserContentMethod::Write,
                    BrowserContentStage::Write,
                    source,
                )
            })?;
        let promise = self.invoke_content(&stream, ContentRequest::Close)?;
        JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    BrowserContentMethod::Close,
                    BrowserContentStage::Close,
                    source,
                )
            })?;
        Ok(())
    }
}

enum ContentRequest<'a> {
    File,
    Buffer,
    Writable,
    Write(&'a FileContents),
    Close,
}

impl WebFile {
    fn invoke_content(
        &self,
        receiver: &JsValue,
        request: ContentRequest<'_>,
    ) -> Result<js_sys::Promise, FileContentError> {
        let method_kind = match request {
            ContentRequest::File => BrowserContentMethod::File,
            ContentRequest::Buffer => BrowserContentMethod::Buffer,
            ContentRequest::Writable => BrowserContentMethod::Writable,
            ContentRequest::Write(_) => BrowserContentMethod::Write,
            ContentRequest::Close => BrowserContentMethod::Close,
        };
        let method = match request {
            ContentRequest::File => "getFile",
            ContentRequest::Buffer => "arrayBuffer",
            ContentRequest::Writable => "createWritable",
            ContentRequest::Write(_) => "write",
            ContentRequest::Close => "close",
        };
        let method = js_sys::Reflect::get(receiver, &method.into())
            .map_err(|source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    method_kind,
                    BrowserContentStage::ReadMethod,
                    source,
                )
            })?
            .dyn_into::<js_sys::Function>()
            .map_err(|source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    method_kind,
                    BrowserContentStage::DecodeMethod,
                    source,
                )
            })?;
        let result = match request {
            ContentRequest::Write(contents) => {
                method.call1(receiver, &js_sys::Uint8Array::from(contents.as_ref()))
            }
            _ => method.call0(receiver),
        }
        .map_err(|source: JsValue| -> FileContentError {
            FileContentError::from_browser(
                self,
                method_kind,
                BrowserContentStage::InvokeMethod,
                source,
            )
        })?;
        result
            .dyn_into::<js_sys::Promise>()
            .map_err(|source: JsValue| -> FileContentError {
                FileContentError::from_browser(
                    self,
                    method_kind,
                    BrowserContentStage::DecodePromise,
                    source,
                )
            })
    }
}
impl FileContentError {
    fn from_browser(
        file: &WebFile,
        method: BrowserContentMethod,
        stage: BrowserContentStage,
        source: JsValue,
    ) -> Self {
        Self::Browser {
            file: file.clone(),
            method,
            stage,
            source: source.into(),
        }
    }
}

#[cfg(test)]
mod tests;
