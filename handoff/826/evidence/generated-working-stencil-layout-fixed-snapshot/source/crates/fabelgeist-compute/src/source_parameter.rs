//! Native parameter lexing and immutable source-to-role association.

use crate::source_error::{GeneratedFunction, GeneratedProgramError, GeneratedProgramResult};
use std::collections::HashSet;

/// Borrowed lexical fields used only while admitting an algorithm's roles.
pub(crate) struct NativeParameter<'source> {
    pub(crate) declaration: &'source str,
    pub(crate) name: &'source str,
    pub(crate) native_type: &'source str,
}

/// An original native declaration and the role admitted from that declaration.
/// Neither projection can be changed independently after construction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SourceParameter<Role> {
    declaration: String,
    role: Role,
}

impl<'source> NativeParameter<'source> {
    /// Resource parameters are source metadata stripped by the owning algorithm.
    /// Classify the native outer form before projecting its element type.
    pub(crate) fn is_resource(&self) -> bool {
        let native_type = self.native_type.trim();
        let outer = native_type
            .strip_suffix('>')
            .and_then(|body| body.split_once('<'))
            .map(|(outer, _)| outer.trim());
        matches!(
            outer,
            Some(
                "Resource"
                    | "ptr"
                    | "texture_1d"
                    | "texture_2d"
                    | "texture_2d_array"
                    | "texture_3d"
                    | "texture_cube"
                    | "texture_cube_array"
                    | "texture_multisampled_2d"
                    | "texture_storage_1d"
                    | "texture_storage_2d"
                    | "texture_storage_2d_array"
                    | "texture_storage_3d"
            )
        ) || matches!(
            native_type,
            "texture_depth_2d"
                | "texture_depth_2d_array"
                | "texture_depth_cube"
                | "texture_depth_cube_array"
                | "texture_depth_multisampled_2d"
        )
    }

    pub(crate) fn parse(
        arguments: &'source str,
        function: GeneratedFunction,
    ) -> GeneratedProgramResult<Vec<Self>> {
        let mut declarations = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        for (index, character) in arguments.char_indices() {
            match character {
                '<' => depth += 1,
                '>' => depth -= 1,
                ',' if depth == 0 => {
                    declarations.push(&arguments[start..index]);
                    start = index + 1;
                }
                _ => {}
            }
        }
        declarations.push(&arguments[start..]);
        let mut parameters = Vec::new();
        let mut names = HashSet::new();
        for declaration in declarations {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }
            let parts: Vec<_> = declaration.split(':').map(str::trim).collect();
            if parts.len() != 2 {
                return Err(GeneratedProgramError::InvalidParameter {
                    function,
                    declaration: declaration.to_owned(),
                });
            }
            let name = parts[0];
            if !names.insert(name) {
                return Err(GeneratedProgramError::DuplicateParameter {
                    function,
                    name: name.to_owned(),
                });
            }
            parameters.push(Self {
                declaration,
                name,
                native_type: parts[1],
            });
        }
        Ok(parameters)
    }
}

impl<Role> SourceParameter<Role> {
    pub(crate) fn new(declaration: &str, role: Role) -> Self {
        Self {
            declaration: declaration.to_owned(),
            role,
        }
    }

    pub(crate) fn declaration(&self) -> &str {
        &self.declaration
    }

    pub(crate) fn role(&self) -> &Role {
        &self.role
    }
}
