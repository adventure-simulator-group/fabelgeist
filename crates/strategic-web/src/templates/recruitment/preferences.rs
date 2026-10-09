//! Optional preferences for a recruitment role.
use super::{combat_requirements, numeric_requirement};
use maud::{Markup, html};
pub(super) fn preferences() -> Markup {
    html! {
                        details class="role-preferences" {
                            summary { "Preferred abilities (optional)" }
                        div class="role-requirements-heading" {
                            h3 { "Individual recommendations" }
                            p { "Applicants may still request to join if they fall short." }
                        }
                        div class="role-requirement-columns role-requirement-columns-individual" {
                            (combat_requirements())
                            div class="role-requirement-group" {
                                header class="role-requirement-heading" {
                                    h3 { "Mobility" }
                                    p { "Movement and sustained physical capability" }
                                }
                                (numeric_requirement("athletics", "Athletics"))
                                (numeric_requirement("endurance", "Endurance"))
                            }
                        }
                        }
    }
}
