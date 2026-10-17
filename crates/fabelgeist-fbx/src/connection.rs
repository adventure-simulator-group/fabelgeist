//! Property roles assigned when an FBX connection is decoded.
use crate::Prop;

/// The model transform addressed by an animation curve node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransformProperty {
    Translation,
    Rotation,
    Scaling,
}

/// The component addressed by a scalar animation curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CurveAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ConnectionRole {
    Transform(TransformProperty),
    Axis(CurveAxis),
    Uninterpreted,
}

/// A decoded connection property, retaining its exact lossy UTF-8 spelling.
///
/// Known animation roles are classified once at admission. Unrecognized names
/// remain available for presentation and graph traversal; they do not become
/// transform or axis targets.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FbxConnectionProperty {
    spelling: String,
    role: ConnectionRole,
}

impl FbxConnectionProperty {
    pub fn transform(&self) -> Option<TransformProperty> {
        match self.role {
            ConnectionRole::Transform(property) => Some(property),
            _ => None,
        }
    }

    pub fn axis(&self) -> Option<CurveAxis> {
        match self.role {
            ConnectionRole::Axis(axis) => Some(axis),
            _ => None,
        }
    }

    pub(crate) fn from_property(property: &Prop) -> Option<Self> {
        match property {
            Prop::Str(bytes) | Prop::Raw(bytes) => Some(Self::from(bytes.as_slice())),
            _ => None,
        }
    }
}

impl From<&[u8]> for FbxConnectionProperty {
    fn from(encoded: &[u8]) -> Self {
        let spelling = String::from_utf8_lossy(encoded).into_owned();
        let role = match spelling.as_str() {
            "Lcl Translation" => ConnectionRole::Transform(TransformProperty::Translation),
            "Lcl Rotation" => ConnectionRole::Transform(TransformProperty::Rotation),
            "Lcl Scaling" => ConnectionRole::Transform(TransformProperty::Scaling),
            "d|X" => ConnectionRole::Axis(CurveAxis::X),
            "d|Y" => ConnectionRole::Axis(CurveAxis::Y),
            "d|Z" => ConnectionRole::Axis(CurveAxis::Z),
            _ => ConnectionRole::Uninterpreted,
        };
        Self { spelling, role }
    }
}

impl std::fmt::Display for FbxConnectionProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.spelling)
    }
}
