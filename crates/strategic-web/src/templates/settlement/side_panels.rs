//! Independent document tabs which retain their authorized content in the DOM.

use maud::{Markup, html};

use crate::spacetimedb::{CharacterView, SettlementView};

use super::Storefront;

pub(super) struct SidePanel<'a> {
    pub side: PanelSide,
    pub subject: PanelSubject<'a>,
    pub tabs: Vec<PanelTab>,
}

pub(super) struct PanelTab {
    pub kind: PanelKind,
    pub content: Markup,
}

#[derive(Clone, Copy)]
pub(super) enum PanelSide {
    Left,
    Right,
}

pub(super) enum PanelSubject<'a> {
    Character(&'a CharacterView),
    Storefront {
        settlement: &'a SettlementView,
        shop: Storefront,
    },
}

#[derive(Clone, Copy)]
pub(super) enum PanelKind {
    Attributes,
    Character,
    Inventory,
    Services,
    Skills,
    Trade,
}

impl SidePanel<'_> {
    pub fn render(self) -> Markup {
        let side = self.side.id();
        let context = self.subject.context();
        html! {
            div class="side-panel" data-side-panel=(side) data-panel-context=(&context) {
                nav class="side-panel-tabs" role="tablist"
                    aria-label=(format!("{} panel", self.side.label())) {
                    @for (index, tab) in self.tabs.iter().enumerate() {
                        @let id = tab.kind.id();
                        button type="button" role="tab" class="side-panel-tab"
                            id=(format!("{side}-{context}-{id}-tab"))
                            aria-controls=(format!("{side}-{context}-{id}-panel"))
                            aria-selected=(if index == 0 { "true" } else { "false" })
                            tabindex=(if index == 0 { "0" } else { "-1" })
                            data-panel-tab=(id) data-strategic-tooltip=(tab.kind.label()) {
                            (tab.kind.label())
                        }
                    }
                }
                @for (index, tab) in self.tabs.into_iter().enumerate() {
                    @let id = tab.kind.id();
                    section class="side-panel-content" role="tabpanel"
                        id=(format!("{side}-{context}-{id}-panel"))
                        aria-labelledby=(format!("{side}-{context}-{id}-tab"))
                        data-panel-pane=(id) hidden[index != 0] tabindex="0" {
                        (tab.content)
                    }
                }
            }
        }
    }
}

impl PanelSide {
    fn id(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}

impl PanelSubject<'_> {
    fn context(&self) -> String {
        match self {
            Self::Character(character) => format!("character-{}", character.id),
            Self::Storefront { settlement, shop } => {
                format!("storefront-{}-{}", settlement.id, shop.service_id())
            }
        }
    }
}

impl PanelKind {
    fn id(self) -> &'static str {
        match self {
            Self::Attributes => "attributes",
            Self::Character => "character",
            Self::Inventory => "inventory",
            Self::Services => "services",
            Self::Skills => "skills",
            Self::Trade => "trade",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Attributes => "Attributes",
            Self::Character => "Character",
            Self::Inventory => "Inventory",
            Self::Services => "Services",
            Self::Skills => "Skills",
            Self::Trade => "Trade",
        }
    }
}

/// Only the already disclosed Character projection is rendered here.
pub(super) fn character_identity(character: &CharacterView) -> Markup {
    html! {
        (crate::templates::sidebar_section(&character.name, html! {
            dl class="social-biography" {
                div { dt { "Age" } dd { (character.age_years) } }
                div { dt { "Level" } dd { (character.level) } }
                div { dt { "Experience" } dd { (character.xp) } }
                div { dt { "Condition" } dd { @if character.alive { "Alive" } @else { "Dead" } } }
            }
        }))
    }
}
