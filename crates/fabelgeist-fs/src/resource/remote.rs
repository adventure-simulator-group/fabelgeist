use super::{ResourceFailure, ResourceLocator};
use crate::FileContents;

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct FailedBlobFetches {
    failures: std::sync::Mutex<
        std::collections::HashMap<ResourceLocator, std::sync::Arc<ResourceFailure>>,
    >,
}
#[cfg(target_arch = "wasm32")]
impl FailedBlobFetches {
    fn prior(&self, locator: &ResourceLocator) -> Option<std::sync::Arc<ResourceFailure>> {
        self.failures.lock().ok()?.get(locator).cloned()
    }
    fn remember(&self, locator: ResourceLocator, cause: std::sync::Arc<ResourceFailure>) {
        if let Ok(mut failures) = self.failures.lock() {
            failures.insert(locator, cause);
        }
    }
}
impl ResourceLocator {
    pub(super) async fn read_remote(&self) -> Result<FileContents, ResourceFailure> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let response = reqwest::get(&self.0)
                .await
                .map_err(ResourceFailure::HttpRequest)?;
            let body = response.bytes().await.map_err(ResourceFailure::HttpBody)?;
            Ok(FileContents::from(body.to_vec()))
        }
        #[cfg(target_arch = "wasm32")]
        {
            use super::error::FetchFailureOrigin;
            use std::sync::{Arc, OnceLock};
            static FAILED_FETCHES: OnceLock<FailedBlobFetches> = OnceLock::new();
            if let Some(cache) = FAILED_FETCHES.get()
                && let Some(cause) = cache.prior(self)
            {
                return Err(ResourceFailure::RetainedFetch {
                    source: cause.clone(),
                    origin: FetchFailureOrigin::Cache,
                });
            }
            let (tx, rx) =
                futures_channel::oneshot::channel::<Result<FileContents, ResourceFailure>>();
            let locator = self.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = locator.fetch_browser().await;
                let result = match result {
                    Err(cause) if locator.0.starts_with("blob:") => {
                        let cause = Arc::new(cause);
                        let cache = FAILED_FETCHES.get_or_init(FailedBlobFetches::default);
                        cache.remember(locator, cause.clone());
                        Err(ResourceFailure::RetainedFetch {
                            source: cause,
                            origin: FetchFailureOrigin::Attempt,
                        })
                    }
                    result => result,
                };
                let _ = tx.send(result);
            });
            rx.await.map_err(ResourceFailure::FetchChannelClosed)?
        }
    }
    #[cfg(target_arch = "wasm32")]
    async fn fetch_browser(&self) -> Result<FileContents, ResourceFailure> {
        use super::error::{BrowserFetchStage, HttpResponseStatus};
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        let options = web_sys::RequestInit::new();
        options.set_method("GET");
        options.set_mode(web_sys::RequestMode::Cors);
        let request = web_sys::Request::new_with_str_and_init(&self.0, &options).map_err(
            |source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::Request,
                    source: source.into(),
                }
            },
        )?;
        let promise = self.request_browser(&request)?;
        let value =
            JsFuture::from(promise)
                .await
                .map_err(|source: JsValue| -> ResourceFailure {
                    ResourceFailure::Browser {
                        stage: BrowserFetchStage::Fetch,
                        source: source.into(),
                    }
                })?;
        let response = value.dyn_into::<web_sys::Response>().map_err(
            |source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::DecodeResponse,
                    source: source.into(),
                }
            },
        )?;
        if !response.ok() {
            return Err(ResourceFailure::HttpStatus(HttpResponseStatus::from(
                response.status(),
            )));
        }
        let promise = response
            .array_buffer()
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::RequestBuffer,
                    source: source.into(),
                }
            })?;
        let buffer = JsFuture::from(promise)
            .await
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::AwaitBuffer,
                    source: source.into(),
                }
            })?
            .dyn_into::<js_sys::ArrayBuffer>()
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::DecodeBuffer,
                    source: source.into(),
                }
            })?;
        Ok(FileContents::from(
            js_sys::Uint8Array::new(&buffer).to_vec(),
        ))
    }
    #[cfg(target_arch = "wasm32")]
    fn request_browser(
        &self,
        request: &web_sys::Request,
    ) -> Result<js_sys::Promise, ResourceFailure> {
        use super::error::BrowserFetchStage;
        use wasm_bindgen::{JsCast, JsValue};
        let window = web_sys::window().ok_or(ResourceFailure::MissingWindow)?;
        let method = js_sys::Reflect::get(&window, &"fetch".into())
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::ReadFetch,
                    source: source.into(),
                }
            })?
            .dyn_into::<js_sys::Function>()
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::DecodeFetch,
                    source: source.into(),
                }
            })?;
        let value =
            method
                .call1(&window, request)
                .map_err(|source: JsValue| -> ResourceFailure {
                    ResourceFailure::Browser {
                        stage: BrowserFetchStage::InvokeFetch,
                        source: source.into(),
                    }
                })?;
        value
            .dyn_into::<js_sys::Promise>()
            .map_err(|source: JsValue| -> ResourceFailure {
                ResourceFailure::Browser {
                    stage: BrowserFetchStage::DecodePromise,
                    source: source.into(),
                }
            })
    }
}
