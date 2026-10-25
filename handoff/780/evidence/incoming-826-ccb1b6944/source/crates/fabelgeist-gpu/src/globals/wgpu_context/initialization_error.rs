use std::error::Error;
use std::fmt;
use web_sys::wasm_bindgen::JsValue;

/// Browser presentation setup failures not represented by a native SDK error.
#[derive(Debug)]
pub enum BrowserContextError {
    /// The original JavaScript failure remains available for inspection.
    Canvas {
        cause: JsValue,
    },
    SurfaceConfigurationUnavailable,
}

impl fmt::Display for BrowserContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Canvas { cause } => {
                write!(formatter, "Failed to create OffscreenCanvas: {cause:?}")
            }
            Self::SurfaceConfigurationUnavailable => {
                formatter.write_str("No compatible presentation surface configuration")
            }
        }
    }
}

// JsValue is an opaque provider value, rather than a standard Error source.
// Keep the value in Canvas instead of replacing it with its Display text.
impl Error for BrowserContextError {}
