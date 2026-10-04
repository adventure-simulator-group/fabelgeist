//! Admission failure for dividing medicinal food components.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MedicinalFoodSplitError;

impl std::fmt::Display for MedicinalFoodSplitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Medicinal food split fraction is invalid")
    }
}

impl std::error::Error for MedicinalFoodSplitError {}
