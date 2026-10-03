//! Quality gates for open catalog descriptions against the canonical bestiary.
//!
//! Callers enumerate catalog entries, including additions without named constants.
//! The embedded monster and relation authority supplies the support distribution.
use super::{
    BestiaryCatalogDiagnostic, RegionalContext, ReportDescription,
    ambiguous_description_cardinality, distinguishing_clue_set_count, distribution_summary,
};

const MAX_CURATED_REPORT_BASIS_POINTS: u16 = 9_500;

pub(crate) fn validate_report_descriptions(
    reports: impl IntoIterator<Item = ReportDescription>,
) -> Vec<BestiaryCatalogDiagnostic> {
    let mut errors = Vec::new();
    for report in reports {
        let cardinality = ambiguous_description_cardinality(report);
        if cardinality < 2 {
            errors.push(BestiaryCatalogDiagnostic {
                message: format!("description {report:?} is not ambiguous"),
            });
        }
        let marginals = distribution_summary(report, RegionalContext::NorthernGermany1544);
        if marginals
            .iter()
            .any(|item| item.curated_basis_points > MAX_CURATED_REPORT_BASIS_POINTS)
        {
            errors.push(BestiaryCatalogDiagnostic {
                message: format!("description {report:?} is over-dominant"),
            });
        }
        if cardinality > 1 && distinguishing_clue_set_count(report) < 2 {
            errors.push(BestiaryCatalogDiagnostic {
                message: format!("description {report:?} lacks distinguishing clues"),
            });
        }
    }
    errors
}
