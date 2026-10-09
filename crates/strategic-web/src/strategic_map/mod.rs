//! Environment-rendered map hosts with canonical, observer-admitted HTML links.
use crate::spacetimedb::{BackendCaseSitePin, SettlementView};
use adventuresim_terrain::{RoutePlan, TerrainPack};
use axum::http::{StatusCode, header::CONTENT_TYPE};
use maud::{Markup, PreEscaped, html};
use producer::{MapLocations, MapPresentation};
use std::collections::BTreeSet;

mod controls;
mod markers;
mod producer;

pub(crate) use producer::has_geographic_source;
pub(crate) const DATA_LICENSE_PATH: &str = "/map/data-license";
const DATA_LICENSE: &str = include_str!("../../../../MAP_DATA_LICENSE.md");

#[expect(
    clippy::too_many_arguments,
    reason = "this admission port joins generated records and the selected HTTP destination"
)]
pub fn strategic_map(
    terrain: &TerrainPack,
    settlements: &[SettlementView],
    case_sites: &[BackendCaseSitePin],
    current: &SettlementView,
    connected: &BTreeSet<&str>,
    selected: Option<&str>,
    path: &str,
    route: Option<&RoutePlan>,
) -> Markup {
    let presentation = MapPresentation::capture(
        terrain,
        MapLocations {
            settlements,
            case_sites,
            current,
            connected,
            selected,
            route,
        },
    );
    match presentation.and_then(|presentation| {
        let json =
            serde_json::to_string(&presentation.input).map_err(producer::MapBuildError::Json)?;
        Ok(render(&presentation, &json, &current.name, path))
    }) {
        Ok(markup) => markup,
        Err(cause) => {
            tracing::error!(%cause, "regional map presentation could not be admitted");
            strategic_map_bundle_unavailable()
        }
    }
}

pub fn strategic_map_unavailable(settlement_name: &str) -> Markup {
    html! { section class="strategic-map strategic-map-unavailable" role="status" {
        h2 { "Map unavailable" }
        p { "No geographic position is available for " (settlement_name) "." }
    } }
}

pub fn strategic_map_bundle_unavailable() -> Markup {
    html! { section class="strategic-map strategic-map-unavailable" role="status" {
        h2 { "Map unavailable" }
        p { "Choose a destination to inspect its route and travel options." }
    } }
}

pub async fn data_license() -> (
    StatusCode,
    [(axum::http::HeaderName, &'static str); 1],
    &'static str,
) {
    (
        StatusCode::OK,
        [(CONTENT_TYPE, "text/plain; charset=utf-8")],
        DATA_LICENSE,
    )
}

fn render(presentation: &MapPresentation, json: &str, name: &str, path: &str) -> Markup {
    // Canonical identities are hex-encoded on the wire. Escape the HTML script
    // boundary as well so future text fields cannot close the JSON element.
    let json = json
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    html! {
        section class="strategic-map" data-regional-map tabindex="0" role="region"
            aria-label=(format!("Map around {name}")) aria-describedby="regional-map-instructions" {
            p id="regional-map-instructions" class="sr-only" {
                "Drag or use arrow keys to pan. Use the wheel, plus or minus to zoom. Use Q and E or a right drag to rotate. Home resets the view; F frames the route."
            }
            script type="application/json" data-regional-map-input { (PreEscaped(json)) }
            div class="strategic-map-window" data-map-window aria-hidden="true" {}
            div class="strategic-map-controls" data-map-foreground role="group" aria-label="Map controls" {
                @for control in &controls::CONTROLS {
                    button type="button" class="strategic-map-control" data-map-action=(control.action.as_str())
                        aria-label=(control.label) data-strategic-tooltip=(control.label) { span aria-hidden="true" { (control.glyph) } }
                }
                button type="button" class="strategic-map-control" data-map-action="retry" hidden
                    aria-label="Retry map" data-strategic-tooltip="Retry map" { span aria-hidden="true" { "↻" } }
            }
            div class="strategic-map-markers" {
                @for link in &presentation.links {
                    @if let Some(href) = link.href(path) {
                        a class=(format!("map-place-link {}", link.state_class())) href=(href)
                            data-map-place=(link.marker.place.to_string()) data-map-foreground hidden
                            aria-label=(&link.label) data-strategic-tooltip=(&link.label)
                            aria-current=[link.selected().then_some("true")] {
                            span class="map-place-symbol" aria-hidden="true" {
                                @if link.marker.rank == adventuresim_tactical_core::regional_map::MapMarkerRank::CaseSite {
                                    (crate::templates::decorative_game_icon("treasure-map"))
                                } @else { (crate::templates::decorative_game_icon("castle")) }
                            }
                            span { (&link.name) }
                            @if let Some(label) = link.state_label() { small class="map-place-state" { (label) } }
                        }
                    }
                }
            }
            p class="strategic-map-status" role="status" data-map-status data-map-foreground { "Loading map…" }
            details class="strategic-map-key" data-map-foreground {
                summary { "Map key" }
                ul {
                    li { span class="map-key-road" aria-hidden="true" {} "Road" }
                    li { span class="map-key-water" aria-hidden="true" {} "Shipping route or ferry" }
                    li { span class="map-key-winter" aria-hidden="true" {} "Winter route" }
                    li { span class="map-key-inferred" aria-hidden="true" {} "Inferred walking link" }
                    li { span class="map-key-selected" aria-hidden="true" {} "Computed route" }
                    li { span class="map-key-estimated" aria-hidden="true" {} "Estimated route" }
                }
            }
            a class="strategic-map-license-link" data-map-foreground href=(DATA_LICENSE_PATH) rel="license" { "Map data licence" }
        }
    }
}

#[cfg(test)]
mod tests;
