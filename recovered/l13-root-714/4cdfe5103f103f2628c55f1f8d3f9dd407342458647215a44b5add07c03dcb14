//! Canonical HTTP location identities, route patterns, and URL construction.
//!
//! Physical venue slugs belong to `SettlementVenueKind`. Service identifiers
//! and NPC presence identifiers are translated only at the HTTP boundary.

use std::{fmt, str::FromStr};

use adventuresim_core::{durability::RepairService, strategic_place::SettlementVenueKind};

pub mod patterns;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationKind {
    Settlement,
    CaseSite,
}

impl LocationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Settlement => "settlement",
            Self::CaseSite => "case-site",
        }
    }

    pub fn path(self, id: &str) -> String {
        match self {
            Self::Settlement => patterns::SETTLEMENT.url([&id]),
            Self::CaseSite => patterns::CASE_SITE.url([&id]),
        }
    }
}

impl fmt::Display for LocationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LocationKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            value if value == Self::Settlement.as_str() => Ok(Self::Settlement),
            value if value == Self::CaseSite.as_str() => Ok(Self::CaseSite),
            _ => Err(()),
        }
    }
}

/// A registered route with a statically bounded number of path parameters.
#[derive(Clone, Copy)]
pub struct Route<const N: usize>(&'static str);

impl<const N: usize> Route<N> {
    const fn new(pattern: &'static str) -> Self {
        Self(pattern)
    }

    pub const fn pattern(self) -> &'static str {
        self.0
    }

    pub fn url(self, parameters: [&dyn fmt::Display; N]) -> String {
        let mut result = String::new();
        let mut remaining = self.0;
        for parameter in parameters {
            let (prefix, tail) = remaining.split_once('{').expect("route parameter");
            result.push_str(prefix);
            result.push_str(&encode_component(&parameter.to_string()));
            remaining = tail.split_once('}').expect("closed route parameter").1;
        }
        assert!(!remaining.contains('{'), "all route parameters supplied");
        result.push_str(remaining);
        result
    }
}

/// Encode a single path or query component, never a complete URL.
pub fn encode_component(value: &str) -> String {
    use fmt::Write;
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(encoded, "%{byte:02X}").expect("writing to a string");
        }
    }
    encoded
}

/// Append a query value before any fragment, preserving the rest of the URL.
pub fn with_query(path: &str, name: &str, value: &str) -> String {
    let (base, fragment) = path.split_once('#').unwrap_or((path, ""));
    let separator = if base.contains('?') { '&' } else { '?' };
    let mut result = format!(
        "{base}{separator}{}={}",
        encode_component(name),
        encode_component(value)
    );
    if path.contains('#') {
        result.push('#');
        result.push_str(fragment);
    }
    result
}

pub fn service_place(service: &str) -> SettlementVenueKind {
    let location = adventuresim_core::organization::service_npc_location_id(service)
        .expect("known settlement service");
    SettlementVenueKind::from_id(location).expect("service has a physical venue")
}

pub fn place_service(place: &str) -> Option<&'static str> {
    match SettlementVenueKind::from_id(place)? {
        SettlementVenueKind::Market => Some("merchants"),
        SettlementVenueKind::Forge => Some("weapons"),
        SettlementVenueKind::Armoury => Some("armor"),
        SettlementVenueKind::Tailor => Some("clothing"),
        SettlementVenueKind::Herbalist => Some("herbalist"),
        SettlementVenueKind::Inn => Some("inn"),
        SettlementVenueKind::Church => Some("religion"),
        SettlementVenueKind::Bookstore => Some("books"),
        SettlementVenueKind::PublicSquare
        | SettlementVenueKind::Residences
        | SettlementVenueKind::Keep => None,
    }
}

pub fn repair_service(place: &str) -> Option<RepairService> {
    place_service(place).and_then(RepairService::parse)
}

/// NPC presence uses an internal identifier for the square, not a URL alias.
const PUBLIC_SQUARE_PRESENCE: &str = "overview";

pub fn npc_location(place: &str) -> &str {
    if place == SettlementVenueKind::PublicSquare.id() {
        PUBLIC_SQUARE_PRESENCE
    } else {
        place
    }
}

pub fn npc_place(location: &str) -> &str {
    if location == PUBLIC_SQUARE_PRESENCE {
        SettlementVenueKind::PublicSquare.id()
    } else {
        location
    }
}

pub fn service_path(settlement: &str, service: &str) -> String {
    patterns::SETTLEMENT_PLACE.url([&settlement, &service_place(service).id()])
}

/// A navigable physical venue or an exact local organization chapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementPlace<'a> {
    Venue(SettlementVenueKind),
    Chapter(&'a str),
}

impl<'a> SettlementPlace<'a> {
    pub fn parse(settlement: &str, slug: &'a str) -> Option<Self> {
        if let Some(venue) = SettlementVenueKind::from_id(slug) {
            return Some(Self::Venue(venue));
        }
        adventuresim_core::organization::organization_chapter_at(settlement, slug)
            .map(|_| Self::Chapter(slug))
    }
}

/// Reject noncanonical identities before a generic route can dispatch them.
pub async fn require_canonical_location_path(
    axum::extract::Path(parameters): axum::extract::Path<std::collections::HashMap<String, String>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let valid_kind = parameters
        .get("kind")
        .is_none_or(|kind| kind.parse::<LocationKind>().is_ok());
    let valid_place = parameters.get("place").is_none_or(|place| {
        parameters
            .get("id")
            .is_some_and(|id| SettlementPlace::parse(id, place).is_some())
    });
    if !valid_kind || !valid_place {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests;

pub(crate) fn local_redirect_path(value: &str) -> Option<String> {
    (value.starts_with('/') && !value.starts_with("//") && !value.contains('\\'))
        .then(|| value.to_owned())
}

/// Settlement party panels share the physical location root.
pub fn party_path(settlement_id: &str, character_id: u64) -> String {
    patterns::PARTY_PERSONAL.url([&LocationKind::Settlement, &settlement_id, &character_id])
}
