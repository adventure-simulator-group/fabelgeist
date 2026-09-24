//! Persistent orientation for document-like strategic tasks.
use maud::{Markup, html};

pub(super) fn navigation(title: &str) -> Markup {
    html! {
        nav class="workspace-bar" aria-label="Adventurer navigation" {
            strong class="workspace-title" { (title) }
            a href="/" data-workspace-location { "Location" }
            a data-workspace-inventory hidden { "Party inventory" }
            a href="/quests" { "Journal" }
            a href="/characters" { "Switch character" }
            button type="button" class="btn btn-secondary" data-workspace-chat hidden
                aria-expanded="true" { "Conversation" }
        }
    }
}
