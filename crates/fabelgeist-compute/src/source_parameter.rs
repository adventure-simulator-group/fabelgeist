//! Native parameter lexing and immutable source-to-role association.

use anyhow::{Result, bail};
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
    pub(crate) fn parse(arguments: &'source str, function: &str) -> Result<Vec<Self>> {
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
                bail!("Invalid {function} parameter format: {declaration}");
            }
            let name = parts[0];
            if !names.insert(name) {
                bail!("Duplicate {function} parameter: {name}");
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
