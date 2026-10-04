use super::{WebDirectory, WorkspaceError};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspacePermission {
    Granted,
    Restricted(WorkspaceAccessRestriction),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceAccessRestriction {
    Prompt,
    Denied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspacePermissionOperation {
    Query,
    Request,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspacePermissionStage {
    EncodeOptions,
    ReadMethod,
    DecodeMethod,
    InvokeMethod,
    DecodePromise,
    AwaitResult,
    DecodeResult,
}

impl WebDirectory {
    pub(super) async fn restore_permission(&self) -> Result<WorkspacePermission, WorkspaceError> {
        match self.permission(WorkspacePermissionOperation::Query).await? {
            WorkspacePermission::Granted => Ok(WorkspacePermission::Granted),
            WorkspacePermission::Restricted(_) => {
                self.permission(WorkspacePermissionOperation::Request).await
            }
        }
    }

    async fn permission(
        &self,
        operation: WorkspacePermissionOperation,
    ) -> Result<WorkspacePermission, WorkspaceError> {
        let promise = self.permission_promise(operation)?;
        let value = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> WorkspaceError {
                WorkspaceError::from_permission(
                    self,
                    operation,
                    WorkspacePermissionStage::AwaitResult,
                    source,
                )
            })?;
        WorkspacePermission::from_browser(self, operation, value)
    }

    fn permission_promise(
        &self,
        operation: WorkspacePermissionOperation,
    ) -> Result<js_sys::Promise, WorkspaceError> {
        let options = js_sys::Object::new();
        let encoded = js_sys::Reflect::set(&options, &"mode".into(), &"read".into()).map_err(
            |source: JsValue| -> WorkspaceError {
                WorkspaceError::from_permission(
                    self,
                    operation,
                    WorkspacePermissionStage::EncodeOptions,
                    source,
                )
            },
        )?;
        if !encoded {
            return Err(WorkspaceError::from_permission(
                self,
                operation,
                WorkspacePermissionStage::EncodeOptions,
                JsValue::FALSE,
            ));
        }
        let method = match operation {
            WorkspacePermissionOperation::Query => "queryPermission",
            WorkspacePermissionOperation::Request => "requestPermission",
        };
        let method = js_sys::Reflect::get(self.as_ref(), &method.into()).map_err(
            |source: JsValue| -> WorkspaceError {
                WorkspaceError::from_permission(
                    self,
                    operation,
                    WorkspacePermissionStage::ReadMethod,
                    source,
                )
            },
        )?;
        let method =
            method
                .dyn_into::<js_sys::Function>()
                .map_err(|source: JsValue| -> WorkspaceError {
                    WorkspaceError::from_permission(
                        self,
                        operation,
                        WorkspacePermissionStage::DecodeMethod,
                        source,
                    )
                })?;
        let value =
            method
                .call1(self.as_ref(), &options)
                .map_err(|source: JsValue| -> WorkspaceError {
                    WorkspaceError::from_permission(
                        self,
                        operation,
                        WorkspacePermissionStage::InvokeMethod,
                        source,
                    )
                })?;
        value
            .dyn_into::<js_sys::Promise>()
            .map_err(|source: JsValue| -> WorkspaceError {
                WorkspaceError::from_permission(
                    self,
                    operation,
                    WorkspacePermissionStage::DecodePromise,
                    source,
                )
            })
    }
}

impl WorkspacePermission {
    fn from_browser(
        directory: &WebDirectory,
        operation: WorkspacePermissionOperation,
        value: JsValue,
    ) -> Result<Self, WorkspaceError> {
        match value.as_string().as_deref() {
            Some("granted") => Ok(Self::Granted),
            Some("prompt") => Ok(Self::Restricted(WorkspaceAccessRestriction::Prompt)),
            Some("denied") => Ok(Self::Restricted(WorkspaceAccessRestriction::Denied)),
            _ => Err(WorkspaceError::from_permission(
                directory,
                operation,
                WorkspacePermissionStage::DecodeResult,
                value,
            )),
        }
    }
}
impl WorkspaceError {
    fn from_permission(
        directory: &WebDirectory,
        operation: WorkspacePermissionOperation,
        stage: WorkspacePermissionStage,
        source: JsValue,
    ) -> Self {
        Self::Permission {
            directory: directory.clone(),
            operation,
            stage,
            source: crate::BrowserIoCause::from(source),
        }
    }
}

#[cfg(test)]
mod tests;
