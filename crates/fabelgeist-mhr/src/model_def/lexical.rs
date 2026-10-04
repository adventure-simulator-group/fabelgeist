//! Admission of model-definition lines and tokens from the external text format.
use super::identity::{JointParameterChannel, ModelParameterName, ParameterSetName};
use fabelgeist_fs::FileText;
use fabelgeist_rig::RigJointName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinitionLine {
    number: usize,
    pub(super) text: String,
}
impl From<(usize, &str)> for DefinitionLine {
    fn from((number, text): (usize, &str)) -> Self {
        let text = match text.split_once('#') {
            Some((prefix, _)) => prefix,
            None => text,
        };
        Self {
            number,
            text: text.trim().to_owned(),
        }
    }
}
impl std::fmt::Display for DefinitionLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.number as u128 + 1, self.text)
    }
}
#[derive(Clone, Debug, Default)]
pub(super) struct DefinitionLines(pub(super) Vec<DefinitionLine>);
impl From<&FileText> for DefinitionLines {
    fn from(text: &FileText) -> Self {
        let mut lines = Vec::new();
        for (ordinal, line) in text.as_ref().lines().enumerate() {
            let line = DefinitionLine::from((ordinal, line));
            if !line.text.is_empty() {
                lines.push(line);
            }
        }
        Self(lines)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum DefinitionSection {
    Transform,
    Sets,
    Limits,
    Unsupported,
}
pub(super) enum LineKind {
    Header,
    Section(DefinitionSection),
    Content,
}
impl DefinitionLine {
    pub(super) fn kind(&self) -> LineKind {
        if self.text == "Momentum Model Definition V1.0" {
            return LineKind::Header;
        }
        if self.text.starts_with('[') && self.text.ends_with(']') {
            let section = match &self.text[1..self.text.len() - 1] {
                "ParameterTransform" => DefinitionSection::Transform,
                "ParameterSets" => DefinitionSection::Sets,
                "Limits" => DefinitionSection::Limits,
                _ => DefinitionSection::Unsupported,
            };
            LineKind::Section(section)
        } else {
            LineKind::Content
        }
    }
    pub(super) fn assignment(&self) -> Option<(DefinitionToken<'_>, DefinitionToken<'_>)> {
        let (target, expression) = self.text.split_once('=')?;
        Some((
            DefinitionToken::from(target),
            DefinitionToken::from(expression),
        ))
    }
    pub(super) fn token(&self) -> DefinitionToken<'_> {
        DefinitionToken::from(self.text.as_str())
    }
    pub(super) fn bounds(&self) -> Option<(DefinitionToken<'_>, DefinitionToken<'_>)> {
        let open = self.text.find('[')?;
        let close = self.text.find(']')?;
        if close <= open {
            return None;
        }
        Some((
            DefinitionToken::from(&self.text[open + 1..close]),
            DefinitionToken::from(&self.text[close + 1..]),
        ))
    }
}
#[derive(Clone, Copy)]
pub(super) struct DefinitionToken<'a>(pub(super) &'a str);
impl<'a> From<&'a str> for DefinitionToken<'a> {
    fn from(text: &'a str) -> Self {
        Self(text.trim())
    }
}
#[derive(Clone, Copy)]
pub(super) enum TokenSeparator {
    Terms,
    Factors,
    Words,
    Bounds,
}
pub(super) enum Directive {
    ParameterSet,
    Limit,
    MinMax,
    Other,
}
pub(super) enum JointSpellingContext {
    Target,
    Reference,
}
impl<'a> DefinitionToken<'a> {
    pub(super) fn directive(self) -> Directive {
        match self.0 {
            "parameterset" => Directive::ParameterSet,
            "limit" => Directive::Limit,
            "minmax" => Directive::MinMax,
            _ => Directive::Other,
        }
    }
    pub(super) fn parameter_name(self) -> ModelParameterName {
        ModelParameterName::from(self.0)
    }
    pub(super) fn set_name(self) -> ParameterSetName {
        ParameterSetName::from(self.0)
    }
    pub(super) fn joint_channel(
        self,
        context: JointSpellingContext,
    ) -> Option<(RigJointName, DefinitionToken<'a>)> {
        let (joint, channel) = self.0.split_once('.')?;
        Some(match context {
            JointSpellingContext::Target => (
                RigJointName::from(joint.trim()),
                DefinitionToken::from(channel),
            ),
            JointSpellingContext::Reference => {
                (RigJointName::from(joint), DefinitionToken(channel))
            }
        })
    }
    pub(super) fn channel(self) -> Option<JointParameterChannel> {
        Some(match self.0 {
            "tx" => JointParameterChannel::TranslationX,
            "ty" => JointParameterChannel::TranslationY,
            "tz" => JointParameterChannel::TranslationZ,
            "rx" => JointParameterChannel::RotationX,
            "ry" => JointParameterChannel::RotationY,
            "rz" => JointParameterChannel::RotationZ,
            "sc" => JointParameterChannel::Scale,
            _ => return None,
        })
    }
    pub(super) fn tokens(self, separator: TokenSeparator) -> DefinitionTokens<'a> {
        let mut tokens = Vec::new();
        for token in self.0.split(|c: char| -> bool {
            match separator {
                TokenSeparator::Terms => c == '+',
                TokenSeparator::Factors => c == '*',
                TokenSeparator::Words => matches!(c, ' ' | '\t'),
                TokenSeparator::Bounds => c == ',',
            }
        }) {
            let token = token.trim();
            if !token.is_empty() {
                tokens.push(DefinitionToken::from(token));
            }
        }
        DefinitionTokens(tokens)
    }
}
pub(super) struct DefinitionTokens<'a>(pub(super) Vec<DefinitionToken<'a>>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedDefinitionToken(String);
impl From<DefinitionToken<'_>> for RejectedDefinitionToken {
    fn from(token: DefinitionToken<'_>) -> Self {
        Self(token.0.to_owned())
    }
}
impl std::fmt::Display for RejectedDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
