//! Closed feedback tokens admitted from URLs and rendered at presentation.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ForageFeedback {
    Location,
    Targets,
    Duration,
    Unavailable,
}

impl ForageFeedback {
    pub(super) fn from_http(code: &str) -> Option<Self> {
        match code {
            "location" => Some(Self::Location),
            "targets" => Some(Self::Targets),
            "duration" => Some(Self::Duration),
            "unavailable" => Some(Self::Unavailable),
            _ => None,
        }
    }

    pub(super) fn message(self) -> &'static str {
        match self {
            Self::Location => "Foraging is unavailable at this location.",
            Self::Targets => "Choose valid forage sources and try again.",
            Self::Duration => "Choose a valid search duration and try again.",
            Self::Unavailable => "The search could not be completed.",
        }
    }
}

impl std::fmt::Display for ForageFeedback {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Location => "location",
            Self::Targets => "targets",
            Self::Duration => "duration",
            Self::Unavailable => "unavailable",
        })
    }
}
