//! The browser computes the same complete-payload identity asynchronously.
//! Input-copy lifetime follows the WebCrypto digest algorithm:
//! <https://w3c.github.io/webcrypto/#SubtleCrypto-method-digest>.

use js_sys::{ArrayBuffer, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::prepared::{LIBRARY_DIGEST_BYTES, LibraryId, Result};

const ALGORITHM: &str = "SHA-256";

/// Platform admission failures never select a second digest implementation.
#[derive(Debug, thiserror::Error)]
pub enum BrowserIdentityFailure {
    #[error("browser SHA-256 is unavailable")]
    Unavailable,
    #[error("browser SHA-256 invocation failed")]
    Invocation,
    #[error("browser SHA-256 promise rejected")]
    Rejected,
    #[error("browser SHA-256 returned a non-ArrayBuffer result")]
    ResultType,
    #[error("browser SHA-256 returned an invalid digest length")]
    ResultLength,
}

pub(super) async fn library_id(bytes: &[u8]) -> Result<LibraryId> {
    let window = web_sys::window().ok_or(BrowserIdentityFailure::Unavailable)?;
    let crypto = window
        .crypto()
        .map_err(|_| BrowserIdentityFailure::Unavailable)?;
    let subtle = Reflect::get(crypto.as_ref(), &JsValue::from_str("subtle"))
        .map_err(|_| BrowserIdentityFailure::Unavailable)?
        .dyn_into::<web_sys::SubtleCrypto>()
        .map_err(|_| BrowserIdentityFailure::Unavailable)?;
    // The safe slice adapter is consumed by digest immediately. WebCrypto
    // copies BufferSource bytes before returning its promise; no live Wasm
    // memory view is retained across this await.
    let promise = subtle
        .digest_with_str_and_u8_array(ALGORITHM, bytes)
        .map_err(|_| BrowserIdentityFailure::Invocation)?;
    let buffer = JsFuture::from(promise)
        .await
        .map_err(|_| BrowserIdentityFailure::Rejected)?
        .dyn_into::<ArrayBuffer>()
        .map_err(|_| BrowserIdentityFailure::ResultType)?;
    let array = Uint8Array::new(&buffer);
    let mut digest = [0; LIBRARY_DIGEST_BYTES];
    if array.length() as usize != digest.len() {
        return Err(BrowserIdentityFailure::ResultLength.into());
    }
    array.copy_to(&mut digest);
    Ok(LibraryId(digest))
}
