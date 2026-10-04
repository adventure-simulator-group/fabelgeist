//! Fail closed when the census cannot read or parse a handwritten source file.
use std::{
    error::Error,
    fmt,
    path::{PathBuf, StripPrefixError},
};

#[derive(Debug)]
pub(crate) enum CensusError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: syn::Error,
    },
    OutsideRoot {
        path: PathBuf,
        source: StripPrefixError,
    },
    Walk(walkdir::Error),
}

impl fmt::Display for CensusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
            Self::OutsideRoot { path, source } => write!(
                formatter,
                "source outside root {}: {source}",
                path.display()
            ),
            Self::Walk(source) => write!(formatter, "cannot enumerate sources: {source}"),
        }
    }
}

impl Error for CensusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::OutsideRoot { source, .. } => Some(source),
            Self::Walk(source) => Some(source),
        }
    }
}
