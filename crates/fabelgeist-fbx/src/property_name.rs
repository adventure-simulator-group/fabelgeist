//! Exact byte identities in FBX property tables and known animation roles.
use crate::Prop;

/// An exact property-table key. Unlike record and class names, it is not
/// normalized or lossily decoded before lookup.
///
/// ```compile_fail
/// use fabelgeist_fbx::{FbxRecordName, Node};
/// let node = Node { name: FbxRecordName::MODEL, props: Vec::new(), children: Vec::new() };
/// node.property70(&FbxRecordName::PROPERTIES70);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FbxPropertyName<'a>(&'a [u8]);

impl FbxPropertyName<'static> {
    pub const COLLISION_TYPE: Self = Self(b"col_type");
    pub const ROTATION_ORDER: Self = Self(b"RotationOrder");
    pub const LOCAL_ROTATION: Self = Self(b"Lcl Rotation");
    pub const PRE_ROTATION: Self = Self(b"PreRotation");
    pub const LOCAL_TRANSLATION: Self = Self(b"Lcl Translation");
    pub const LOCAL_SCALING: Self = Self(b"Lcl Scaling");
    pub const LOCAL_START: Self = Self(b"LocalStart");
    pub const LOCAL_STOP: Self = Self(b"LocalStop");
    pub const CURVE_X: Self = Self(b"d|X");
    pub const CURVE_Y: Self = Self(b"d|Y");
    pub const CURVE_Z: Self = Self(b"d|Z");
}
impl<'a> From<&'a [u8]> for FbxPropertyName<'a> {
    fn from(encoded: &'a [u8]) -> Self {
        Self(encoded)
    }
}
impl<'a> FbxPropertyName<'a> {
    pub(crate) fn from_property(property: &'a Prop) -> Option<Self> {
        match property {
            Prop::Str(bytes) | Prop::Raw(bytes) => Some(Self::from(bytes.as_slice())),
            _ => None,
        }
    }
    pub fn transform(self) -> Option<TransformProperty> {
        if self == FbxPropertyName::LOCAL_TRANSLATION {
            Some(TransformProperty::Translation)
        } else if self == FbxPropertyName::LOCAL_ROTATION {
            Some(TransformProperty::Rotation)
        } else if self == FbxPropertyName::LOCAL_SCALING {
            Some(TransformProperty::Scaling)
        } else {
            None
        }
    }
    pub fn axis(self) -> Option<CurveAxis> {
        if self == FbxPropertyName::CURVE_X {
            Some(CurveAxis::X)
        } else if self == FbxPropertyName::CURVE_Y {
            Some(CurveAxis::Y)
        } else if self == FbxPropertyName::CURVE_Z {
            Some(CurveAxis::Z)
        } else {
            None
        }
    }
}
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
