//! Explicit ordinary-scene entry while strategic navigation remains available.

use crate::spacetimedb::CharacterView;
use maud::{Markup, html};

pub(super) fn control(character: Option<&CharacterView>, settlement_id: &str) -> Markup {
    html! {
        @if character.is_some_and(|character| character.alive
            && character.current_settlement_id.as_deref() == Some(settlement_id)
            && character.current_case_site_id.is_none()) {
            form method="post" action="/scenes/settlement" {
                button type="submit" class="btn btn-primary" { "Enter town" }
            }
        }
    }
}
