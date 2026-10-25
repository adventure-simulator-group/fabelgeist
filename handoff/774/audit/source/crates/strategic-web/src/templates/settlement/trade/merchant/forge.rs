//! Weapon recipe controls rendered in the merchant's native HTML port.

use crate::spacetimedb::SettlementView;
use maud::{Markup, html};

pub(super) fn weapon_forge_controls(settlement: &SettlementView) -> Markup {
    html! {
            section class="sidebar-section forge-customization" data-forge-customization data-live-preserve="forge-customization" {
                h2 { "Forge a weapon" }
                form method="post" action=(crate::location_urls::patterns::FORGE_WEAPON.url([&settlement.id])) {
                    label { "Chassis" select data-forge-catalog aria-label="Weapon chassis" {} }
                    div class="forge-recipe-editor" data-forge-editor aria-live="polite" { "Loading complete weapon recipe…" }
                    input type="hidden" name="recipe" data-forge-recipe;
                    dl class="forge-material-staging" data-forge-materials aria-live="polite" {}
                    div class="forge-submit-row" {
                        button type="submit" class="btn btn-primary" disabled data-forge-submit { "Forge" }
                        span data-forge-eta { "Calculating…" }
                    }
                }
            }
    }
}
