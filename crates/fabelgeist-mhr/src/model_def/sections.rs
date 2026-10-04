//! Whole-file section admission; repeated supported sections retain source order.
use super::error::ModelDefinitionError;
use super::lexical::{DefinitionLines, DefinitionSection, LineKind};
use fabelgeist_fs::FileText;

pub(super) struct DefinitionSections {
    pub(super) transform: DefinitionLines,
    pub(super) sets: DefinitionLines,
    pub(super) limits: DefinitionLines,
}
impl TryFrom<&FileText> for DefinitionSections {
    type Error = ModelDefinitionError;
    fn try_from(text: &FileText) -> Result<Self, ModelDefinitionError> {
        let mut lines = DefinitionLines::from(text).0.into_iter();
        let Some(first) = lines.next() else {
            return Err(ModelDefinitionError::MissingHeader);
        };
        if !matches!(first.kind(), LineKind::Header) {
            return Err(ModelDefinitionError::InvalidHeader(first));
        }
        let mut sections = Self {
            transform: DefinitionLines::default(),
            sets: DefinitionLines::default(),
            limits: DefinitionLines::default(),
        };
        let mut current = DefinitionSection::Unsupported;
        for line in lines {
            match line.kind() {
                LineKind::Section(section) => current = section,
                LineKind::Content | LineKind::Header => match current {
                    DefinitionSection::Transform => sections.transform.0.push(line),
                    DefinitionSection::Sets => sections.sets.0.push(line),
                    DefinitionSection::Limits => sections.limits.0.push(line),
                    DefinitionSection::Unsupported => (),
                },
            }
        }
        Ok(sections)
    }
}
