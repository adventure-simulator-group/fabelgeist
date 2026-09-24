//! Summary of observer-authorized medical readings.
use super::{ChartReadingPresentation, MedicalPresentation, physiology_likelihood};
use maud::{Markup, html};
pub(super) fn overview(
    medical: &MedicalPresentation,
    dialog_id: &str,
    latest: &ChartReadingPresentation,
) -> Markup {
    html! {
                        p class="medical-observation-summary" { (medical.readings.len()) " observations · latest confidence " (latest.confidence_bps / 100) "%. " @if medical.readings.len() == 1 { "One observation cannot establish a trend." } " Possible diseases are estimates, not confirmed diagnoses." }
                            div class="physiology-differential" {
                                div {
                                    strong { "Possible diseases" }
                                    span { "Colour confidence improves with skill and observation." }
                                }
                                ul aria-label="Possible diseases ordered by estimated likelihood" {
                                    @for (candidate_index, candidate) in latest.possible_diseases.iter().enumerate() {
                                        @let tooltip_id = format!(
                                            "{dialog_id}-disease-effects-{candidate_index}"
                                        );
                                        (physiology_likelihood(candidate, &tooltip_id))
                                    }
                                }
                            }
    }
}
