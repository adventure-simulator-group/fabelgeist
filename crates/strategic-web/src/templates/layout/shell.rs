//! Document shell, assets, and persistent renderer lifetime.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ScriptProfile {
    Entry,
    Live,
    Strategic,
}

pub(super) fn page_shell(
    title: &str,
    header: Markup,
    content: Markup,
    scripts: ScriptProfile,
    presentation: Option<SettlementPresentation>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (page_head(title, scripts))
            body {
                @if scripts == ScriptProfile::Strategic {
                    div id="strategic-render-surface" aria-hidden="true" {
                        canvas id="game-canvas" {}
                    }
                }
                @if scripts != ScriptProfile::Entry {
                    div id="strategic-live-stream" data-init="@get('/live')" {
                        span id="strategic-live-revision" data-live-revision="0" hidden {}
                    }
                }
                (maud::PreEscaped("<!-- strategic-page-start -->"))
                div class="app" id="strategic-page" data-page-title=(title)
                    data-architectural-family=[presentation.and_then(|value| value.family.map(|family| family.tag()))]
                    data-place-skin=[presentation.map(|value| value.skin.tag())]
                    data-building-material=[presentation.map(|value| value.material.tag())]
                    style=[presentation.map(|value| format!("--active-building-tint:{}", value.material.tint()))]
                    data-script-profile=(match scripts { ScriptProfile::Entry => "entry", ScriptProfile::Live => "live", ScriptProfile::Strategic => "strategic" }) {
                    (header)
                    @if scripts != ScriptProfile::Entry { (workspace::navigation(title)) }

                    div class="main-grid" {
                        (content)
                    }
                }
                (maud::PreEscaped("<!-- strategic-page-end -->"))
            }
        }
    }
}

fn page_head(title: &str, scripts: ScriptProfile) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { (title) " - Fabelgeist" }

            link rel="stylesheet" href="/static/css/base.css?v=roman-garamond-1";
            // Shared CSS
            link rel="stylesheet" href="/static/css/reset.css?v=roman-garamond-1";
            link rel="stylesheet" href="/static/css/layout.css?v=goslar-1";
            link rel="stylesheet" href="/static/css/components.css?v=roman-garamond-1";
            link rel="stylesheet" href="/static/css/strategic.css?v=goslar-2";
            link rel="stylesheet" href="/static/css/architecture.css?v=goslar-2";
            link rel="stylesheet" href="/static/css/utilities.css?v=roman-garamond-1";
            link rel="stylesheet" href="/static/css/workspace.css?v=1";
            link rel="stylesheet" href="/static/css/readability.css?v=1";

            // Datastar
            script type="module" src="https://cdn.jsdelivr.net/gh/starfederation/datastar/bundles/datastar.js" {}
            script {
                (PreEscaped(format!(
                    "window.strategicCalendar=Object.freeze({{minutesPerDay:{MINUTES_PER_DAY},daysPerYear:{DAYS_PER_YEAR},lunarCycleMinutes:{LUNAR_CYCLE_MINUTES}}});"
                )))
            }
            script src="/static/background-fetch.js?v=background-fetch-2" {}
            script src="/static/location-urls.js?v=location-urls-1" {}
            script src="/static/developer-mode.js?v=development-clock-2" defer {}
            script src="/static/tooltips.js?v=delegated-mouseover-1" defer {}
            script src="/static/character-action-dialog.js?v=character-actions-1" defer {}
            script src="/static/workspace.js?v=1" defer {}
            script src="/static/action-previews.js?v=1" defer {}
            @if scripts != ScriptProfile::Entry {
                script src="/static/live-state.js?v=location-urls-1" defer {}
                script src="/static/live-regions.js?v=location-urls-1" defer {}
            }
            @if scripts == ScriptProfile::Strategic {
                script src="/static/strategic-navigation.js?v=places-scroll-1" defer {}
                script type="module" src="/static/strategic-renderer.js?v=model-owned-forge-controls-1" {}
                script src="/static/strategic-mutations.js?v=formaction-override-1" defer {}
                script src="/static/character-switcher.js?v=multi-character-switcher-1" defer {}
                script src="/static/journal-tab.js?v=journal-tab-1" defer {}
                script src="/static/numeric-editor.js?v=draft-callbacks-3" defer {}
                script src="/static/inventory-browser.js?v=framed-equipment-portraits-1" defer {}
                script src="/static/party-trade.js?v=provision-party-food-1-slot-controls-1" defer {}
                script src="/static/cooking.js?v=fireplace-station-1" defer {}
                script src="/static/herbalism.js?v=bounded-craft-1" defer {}
                script src="/static/equipment-toggle.js?v=location-keyboard-slots-5" defer {}
                script src="/static/party-notifications.js?v=standing-leadership-votes-5" defer {}
            script src="/static/party-recruitment.js?v=party-recruitment-live-3" defer {}
            script src="/static/physiology-dialog.js?v=visual-notebook-2" defer {}
                script src="/static/service-quests.js?v=location-urls-1" defer {}
                script src="/static/dialogue-client.js?v=location-urls-1" defer {}
                script src="/static/physical-evidence.js?v=location-urls-1" defer {}
                script src="/static/developer-quest-editor.js?v=scenario-gallery-1" defer {}
                script src="/static/chat-resize.js?v=counterparty-portraits-1" defer {}
                script src="/static/local-chat.js?v=location-urls-1" defer {}
                script src="/static/strategic-condition.js?v=strategic-condition-4" defer {}
                script src="/static/building-state.js?v=goslar-1" defer {}
                script src="/static/travel-planner.js?v=travel-rails-2" defer {}
                script src="/static/strategic-map.js?v=population-culling-3" defer {}
                script src="/static/rest-duration.js?v=wake-time-5" defer {}
                script src="/static/schedule-preview.js?v=server-preview-1" defer {}
                script src="/static/training-schedule.js?v=server-preview-1" defer {}
                script src="/static/immediate-activity.js?v=manual-activities-2" defer {}
            }
        }
    }
}
