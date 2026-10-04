use super::*;
use crate::{BrowserPickerStage, PickerKind, PickerOptionField};
use wasm_bindgen::JsValue;

impl FilePickerFilter {
    pub(crate) fn browser_options(&self) -> Result<js_sys::Object, PickerError> {
        let options = js_sys::Object::new();
        PickerOptionField::Multiple.encode(&options, &JsValue::FALSE)?;
        if let Self::AcceptedType(filter) = self {
            let types = js_sys::Array::new();
            let file_type = filter.browser_type()?;
            types.push(&file_type);
            PickerOptionField::Types.encode(&options, &types)?;
        }
        Ok(options)
    }
}
impl FileTypeFilter {
    fn browser_type(&self) -> Result<js_sys::Object, PickerError> {
        let file_type = js_sys::Object::new();
        PickerOptionField::Description.encode(&file_type, &self.description.as_str().into())?;
        let accept = js_sys::Object::new();
        let encoded = js_sys::Reflect::set(
            &accept,
            &self.mime.to_string().into(),
            &self.suffixes.browser_extensions(),
        )
        .map_err(|source: JsValue| -> PickerError {
            PickerError::from_browser(
                PickerKind::File,
                BrowserPickerStage::EncodeMime(self.mime),
                source,
            )
        })?;
        if !encoded {
            return Err(PickerError::from_browser(
                PickerKind::File,
                BrowserPickerStage::EncodeMime(self.mime),
                JsValue::FALSE,
            ));
        }
        PickerOptionField::Accept.encode(&file_type, &accept)?;
        Ok(file_type)
    }
}
impl PickerOptionField {
    fn encode(self, target: &js_sys::Object, value: &JsValue) -> Result<(), PickerError> {
        let name = match self {
            Self::Description => "description",
            Self::Accept => "accept",
            Self::Types => "types",
            Self::Multiple => "multiple",
        };
        let encoded = js_sys::Reflect::set(target, &name.into(), value).map_err(
            |source: JsValue| -> PickerError {
                PickerError::from_browser(
                    PickerKind::File,
                    BrowserPickerStage::EncodeOption(self),
                    source,
                )
            },
        )?;
        if encoded {
            Ok(())
        } else {
            Err(PickerError::from_browser(
                PickerKind::File,
                BrowserPickerStage::EncodeOption(self),
                JsValue::FALSE,
            ))
        }
    }
}

#[cfg(test)]
mod tests;
