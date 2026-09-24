//! Recoverable forage-environment failure presentation.
use maud::{Markup, html};
pub(super) fn unavailable(reason: &str, return_to: &str) -> Markup {
    html! {
                    p role="alert" class="badge badge-danger" { (reason) }
                    p { "Nearby food sources are unavailable. Try again, or return to your adventurer." }
                    div class="modal-actions" {
                        button type="button" class="btn btn-primary" onclick="location.reload()" { "Try again" }
                        a class="btn btn-secondary character-action-dialog-close" href=(return_to) { "Return" }
                    }
    }
}
