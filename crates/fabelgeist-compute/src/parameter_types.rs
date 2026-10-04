//! Parsed shader declarations acquire pass lookup identity before resource use.
use fabelgeist_gpu::prelude::{
    PassParameter, PassParameterName, PassParameters, ResourceBaseType, ResourceDescriptor,
};
use std::collections::HashMap;

pub(crate) struct ShaderParameterTypes(HashMap<PassParameterName, ResourceBaseType>);
impl From<&[(String, ResourceBaseType)]> for ShaderParameterTypes {
    fn from(declarations: &[(String, ResourceBaseType)]) -> Self {
        let mut types = HashMap::new();
        for (name, ty) in declarations {
            // The signature parser's ordered lookup used the first declaration.
            types
                .entry(PassParameterName::from(name.as_str()))
                .or_insert_with(|| -> ResourceBaseType { ty.clone() });
        }
        Self(types)
    }
}
impl ShaderParameterTypes {
    pub fn resources(
        &self,
        parameters: &PassParameters,
    ) -> HashMap<PassParameterName, ResourceDescriptor> {
        let mut resources = HashMap::new();
        for (name, parameter) in parameters.iter() {
            let descriptor = match parameter {
                PassParameter::Buffer(_) => {
                    self.0
                        .get(name)
                        .map(|ty: &ResourceBaseType| -> ResourceDescriptor {
                            ResourceDescriptor::Buffer(ty.clone())
                        })
                }
                PassParameter::Texture2d(texture) => {
                    Some(ResourceDescriptor::Texture2d(texture.format))
                }
                PassParameter::Texture3d(texture) => {
                    Some(ResourceDescriptor::Texture3d(texture.format))
                }
                _ => None,
            };
            if let Some(descriptor) = descriptor {
                resources.insert(name.clone(), descriptor);
            }
        }
        resources
    }
}
