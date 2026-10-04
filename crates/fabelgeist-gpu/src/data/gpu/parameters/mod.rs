mod name;
pub mod parameter;
mod scalar;
mod uniform;
use indexmap::IndexMap;
pub use name::PassParameterName;
pub use parameter::*;
pub use scalar::{UniformNumber, UniformUnsigned};
pub use uniform::{UniformBytes, UniformPackingError, UniformPackingPolicy};

/// Named values in insertion order; replacement retains the first position.
#[derive(Clone, Debug, Default)]
pub struct PassParameters {
    parameters: IndexMap<PassParameterName, PassParameter>,
}
impl PassParameters {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, key: PassParameterName, value: PassParameter) {
        self.parameters.insert(key, value);
    }
    pub fn get(&self, key: &PassParameterName) -> Option<&PassParameter> {
        self.parameters.get(key)
    }

    /// Replace supplied names and append new names, retaining existing order.
    pub fn overlay(&mut self, values: Self) {
        self.parameters.extend(values.parameters);
    }
    pub fn iter(&self) -> impl Iterator<Item = (&PassParameterName, &PassParameter)> {
        self.parameters.iter()
    }
}

impl<const N: usize> From<[(PassParameterName, PassParameter); N]> for PassParameters {
    fn from(values: [(PassParameterName, PassParameter); N]) -> Self {
        Self {
            parameters: IndexMap::from(values),
        }
    }
}
