//! Draft ingredients and explicit cooking commitment.
use maud::{Markup, html};
pub(super) fn selection(action_base: &str, inventory_scope: &str, method: &str) -> Markup {
    html! {
        section class="cooking-activity" {
            input type="radio" name="method-preview" value=(method) checked hidden data-cooking-method;
            form id="cooking-submit-form" method="post" action=(format!("{action_base}/ingredients")) {
                input type="hidden" name="inventory_scope" value=(inventory_scope);
                input type="hidden" name="inventory_item_ids" value="" data-cooking-ids;
                input type="hidden" name="fractions_micros" value="" data-cooking-amounts;
                h2 { "Prepare a spit roast" }
                p { "Add food portions from your ingredients. Starting the roast combines them into one meal. Vessels cook their own contents separately." }
                p class="small-copy text-muted cooking-preview" data-cooking-preview { "Stage at least one measured food portion." }
                button type="submit" class="btn btn-primary" disabled data-cook-submit { "Start spit roast" }
            }
            p data-cooking-pot-empty { "No ingredients selected." }
            div data-inventory-browser="cooking-pot-left" { table class="trade-inventory-table" aria-label="Selected ingredients" { tbody {} } }
        }
    }
}
