//! Exact neighborhood source protocol and its admitted native payload.

use crate::prelude::*;
use crate::source_error::{GeneratedProgramError, GeneratedProgramResult};
use fabelgeist_gpu::data::gpu::signature::parse_base_type;

pub(super) mod boundary;
mod coordinates;
mod fetch;
mod grid;
pub(super) mod layout;
mod neighbors;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct NeighborhoodParameter {
    element_type: ResourceBaseType,
    dimension: NeighborhoodDimension,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Number of axes sampled by the reserved neighborhood source protocol.
pub enum NeighborhoodDimension {
    /// A line.
    One,
    /// A plane.
    Two,
    /// A volume.
    Three,
    /// A volume with a fourth sample axis.
    Four,
}

impl NeighborhoodParameter {
    pub(super) fn from_native(
        name: &str,
        native_type: &str,
    ) -> GeneratedProgramResult<Option<Self>> {
        let native_type = native_type.trim();
        let outer = native_type
            .split_once('<')
            .map(|(outer, _)| outer.trim())
            .unwrap_or(native_type);
        let dimension = match outer {
            "Neighbors1D" => NeighborhoodDimension::One,
            "Neighbors2D" => NeighborhoodDimension::Two,
            "Neighbors3D" => NeighborhoodDimension::Three,
            "Neighbors4D" => NeighborhoodDimension::Four,
            _ => {
                if outer
                    .strip_prefix("Neighbors")
                    .and_then(|dimension| dimension.strip_suffix('D'))
                    .is_some_and(|dimension| {
                        !dimension.is_empty()
                            && dimension
                                .chars()
                                .all(|character| character.is_ascii_digit())
                    })
                {
                    return Err(GeneratedProgramError::UnsupportedNeighborhoodType {
                        parameter: name.to_owned(),
                        native_type: native_type.to_owned(),
                    });
                }
                return Ok(None);
            }
        };
        let inner = native_type
            .strip_suffix('>')
            .and_then(|body| body.split_once('<'))
            .map(|(_, inner)| inner.trim())
            .ok_or_else(|| GeneratedProgramError::InvalidNeighborhoodType {
                parameter: name.to_owned(),
                native_type: native_type.to_owned(),
            })?;
        Ok(Some(Self {
            element_type: parse_base_type(inner),
            dimension,
        }))
    }

    pub(super) fn emit(
        &self,
        code: &mut String,
        signature: &super::signature::StencilSignature,
        input: &ResourceDescriptor,
        output: &ResourceDescriptor,
    ) -> Result<TextureAddressing> {
        layout::admit_size(self.dimension(), signature, input)?;
        let fetch = fetch::NeighborhoodFetch::new(self, input)?;
        code.push_str(&grid::declaration(
            self.dimension(),
            signature,
            input,
            output,
        ));
        let addressing = coordinates::emit(code, self.dimension(), output);
        neighbors::emit(code, self, &fetch);
        Ok(addressing)
    }

    pub(super) fn declare(&self, code: &mut String) {
        let element_type = self.element_type().as_str();
        let type_name = sanitize_type_name(&element_type);

        match self.dimension() {
            NeighborhoodDimension::One => {
                code.push_str(&format!(
                        "struct Neighbors1D_{} {{\n    left: {},\n    right: {},\n    prev: {},\n    next: {},\n}};\n\n",
                        type_name, element_type, element_type, element_type, element_type
                    ));
            }
            NeighborhoodDimension::Two => {
                code.push_str(&format!(
                        "struct Neighbors2D_{} {{\n    left: {},\n    right: {},\n    top: {},\n    bottom: {},\n    up: {},\n    down: {},\n}};\n\n",
                        type_name, element_type, element_type, element_type, element_type, element_type, element_type
                    ));
            }
            NeighborhoodDimension::Three => {
                code.push_str(&format!(
                        "struct Neighbors3D_{} {{\n    left: {},\n    right: {},\n    top: {},\n    bottom: {},\n    up: {},\n    down: {},\n    front: {},\n    back: {},\n}};\n\n",
                        type_name, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type
                    ));
            }
            NeighborhoodDimension::Four => {
                code.push_str(&format!(
                        "struct Neighbors4D_{} {{\n    left: {},\n    right: {},\n    top: {},\n    bottom: {},\n    up: {},\n    down: {},\n    front: {},\n    back: {},\n    ana: {},\n    kata: {},\n    w_left: {},\n    w_right: {},\n    past: {},\n    future: {},\n}};\n\n",
                        type_name, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type, element_type
                    ));
            }
        }
    }

    pub(super) fn element_type(&self) -> &ResourceBaseType {
        &self.element_type
    }

    pub(super) fn dimension(&self) -> NeighborhoodDimension {
        self.dimension
    }
}

pub(super) fn sanitize_type_name(native_type: &str) -> String {
    native_type
        .replace('<', "_")
        .replace('>', "")
        .replace(' ', "")
        .replace(',', "_")
}
