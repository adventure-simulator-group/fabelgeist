use super::ResourceBaseType;
use std::{error::Error, fmt};

pub type CoordinateParameterResult<T> = Result<T, CoordinateParameterError>;

/// The native parameter identifier and its admitted coordinate shape.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CoordinateParameter {
    name: String,
    shape: CoordinateShape,
}

#[derive(Debug)]
pub struct CoordinateParameterError {
    name: String,
    native_type: String,
    source: strum::ParseError,
}

/// A supported unsigned coordinate shape in a compute function signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum::EnumString)]
pub enum CoordinateShape {
    #[strum(serialize = "u32")]
    Scalar,
    #[strum(serialize = "vec2<u32>")]
    Plane,
    #[strum(serialize = "vec3<u32>")]
    Volume,
}

impl CoordinateShape {
    pub fn from_native_type(native_type: &str) -> Option<Self> {
        native_type.parse().ok()
    }

    pub fn value_type(self) -> ResourceBaseType {
        match self {
            Self::Scalar => ResourceBaseType::U32,
            Self::Plane => ResourceBaseType::Vec2(Box::new(ResourceBaseType::U32)),
            Self::Volume => ResourceBaseType::Vec3(Box::new(ResourceBaseType::U32)),
        }
    }
}

impl CoordinateParameter {
    pub fn new(name: &str, shape: CoordinateShape) -> Self {
        Self {
            name: name.to_owned(),
            shape,
        }
    }

    pub fn from_native(name: &str, native_type: &str) -> CoordinateParameterResult<Self> {
        let shape = native_type
            .parse()
            .map_err(|source| CoordinateParameterError {
                name: name.to_owned(),
                native_type: native_type.to_owned(),
                source,
            })?;
        Ok(Self::new(name, shape))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn shape(&self) -> CoordinateShape {
        self.shape
    }
}

impl fmt::Display for CoordinateParameterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported coordinate type {} for parameter {}",
            self.native_type, self.name
        )
    }
}

impl Error for CoordinateParameterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
