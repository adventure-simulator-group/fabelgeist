//! Compact, non-hover keys for the visual language of document panels.

use maud::{Markup, html};

pub(super) fn inventory_key(quantities: bool, equipped: bool) -> Markup {
    html! {
        details class="interface-key" {
            summary { "Column key" }
            dl {
                @if quantities {
                    dt { "Have / Target" }
                    dd { "Have is the amount here. Target is the amount you want to retain or acquire in this inventory." }
                }
                @if equipped {
                    dt { "Worn" }
                    dd { "Equipped or attached items. Select an item's equipment control to see named placement choices; letters are keyboard shortcuts." }
                }
                dt { "kg / Value" }
                dd { "Weight in kilograms and currency value. Expand an item for its details." }
                dt { "Columns" }
                dd { "Show weapon and armor measurements with the Columns control. A dash means no reported value, rather than zero." }
            }
        }
    }
}

pub(super) fn skill_key() -> Markup {
    html! {
        (stats_labels("skills"))
        details class="interface-key stats-key" {
            summary { "Skill scale · 0–5" }
            div class="rank-scale-key" aria-label="Rank colors from zero to five" {
                @for rank in 0..=5 {
                    span class="rank-color-swatch" data-rank=(rank)
                        style=(format!("--rank-color:var(--rank-tier-{rank})")) { (rank) }
                }
            }
            p { "Filled segments show current usable rank: farther right means higher skill. Each segment spans one rank. Hatched portions show training above the current usable rank, limited by aptitude or injury." }
            p { "Expand a skill family to see its individual skills. Select an action icon to use that skill." }
        }
    }
}

pub(super) fn body_key() -> Markup {
    html! {
        (stats_labels("attributes"))
        details class="interface-key stats-key" {
            summary { "Reading attributes and health" }
            p { "Attributes use a 0–5 scale. Filled length is current ability; hatching is ability lost to injury. Open a region's health reading for the known breakdown." }
            ul class="interface-swatch-key" {
                @for (class, label) in [
                    ("attribute-health-current", "Sound"),
                    ("attribute-health-cut", "Cut"),
                    ("attribute-health-frostbite", "Frostbite"),
                    ("attribute-health-blunt", "Bruise"),
                    ("attribute-health-fracture", "Fracture"),
                    ("attribute-health-other", "Other impairment"),
                ] {
                    li { i class=(class) aria-hidden="true" {} (label) }
                }
            }
            p { "Banding marks bandaged cuts or splinted fractures. Other impairment is not a diagnosis. Assessed humours are named in the physician notebook." }
        }
    }
}

fn stats_labels(panel: &str) -> Markup {
    html! {
        label class="stats-label-control" {
            input type="checkbox" data-stats-labels=(panel);
            "Show labels"
        }
    }
}

pub(super) fn regional_reading(
    region: crate::spacetimedb::BodyRegion,
    reading: &str,
    sound: f32,
    bar: Markup,
) -> Markup {
    html! {
        details class="region-reading" data-impaired=(sound < 1.0) {
            summary aria-label=(reading) data-strategic-tooltip=(reading) {
                (body_region_icon(region))
                (bar)
                span class="region-health-symbol" aria-hidden="true" { "+" }
            }
            span class="region-health-label" { (format!("{:.0}% sound", sound * 100.0)) }
            p { (reading) }
        }
    }
}

pub(super) fn wheel_key() -> Markup {
    html! {
        details class="interface-key condition-key" {
            summary { "Condition rings" }
            p { "More filled means greater incapacitation; the ring stops at 100%. Black is fatigue, pink pain, red blood loss, blue fear, brown hunger, teal thirst, and violet temperature strain." }
            p { "Grey marks encumbrance when shown. Yellow extensions are projected increases, not current impairment. Open a character's status for the current breakdown." }
        }
    }
}

fn body_region_icon(region: crate::spacetimedb::BodyRegion) -> Markup {
    use crate::spacetimedb::BodyRegion;
    let icon = match region {
        BodyRegion::Head => "anatomical-head",
        BodyRegion::Chest => "muscular-torso",
        BodyRegion::Abdomen => "stomach",
        BodyRegion::LeftArm | BodyRegion::RightArm => "arm",
        BodyRegion::LeftLeg | BodyRegion::RightLeg => "leg",
    };
    html! {
        span class="region-body-icon" data-body-region=(region.slug()) aria-hidden="true" {
            (super::decorative_game_icon(icon))
        }
    }
}
