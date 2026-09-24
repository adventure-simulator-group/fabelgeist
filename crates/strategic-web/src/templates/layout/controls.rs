//! Character and reference controls in the strategic header.
use maud::{Markup, html};

pub(super) fn character_switcher(name: &str) -> Markup {
    let initial = name.chars().next().unwrap_or('?');
    html! {
        button type="button" class="developer-mode-toggle" data-developer-mode-toggle
            aria-label="Enable developer mode" aria-pressed="false" title="Developer mode" {
            span class="developer-mode-icon" aria-hidden="true" {}
        }
        details class="character-switcher" {
            summary class="character-switcher-toggle"
                aria-label=(format!("Character menu for {name}")) title=(name) {
                span class="party-portrait-initial character-switcher-portrait" aria-hidden="true" {
                    span class="party-portrait-face" { (initial) }
                }
            }
            div class="character-switcher-menu" {
                (crate::templates::interface_help::wheel_key())
                div data-character-switcher-options data-character-switcher-url="/characters/menu" {
                    p class="character-switcher-empty" { "Loading adventurers…" }
                }
                a href="/characters/candidates" class="btn btn-small" { "Character select" }
            }
        }
    }
}

pub(super) fn journal_button() -> Markup {
    html! {
        span class="reference-buttons" {
            a href="/quests" class="journal-button" data-journal-tab
                aria-label="Open journal" aria-pressed="false"
                title="Journal" data-strategic-tooltip="Journal" {
                span class="journal-button-icon" aria-hidden="true" {}
            }
        }
    }
}
