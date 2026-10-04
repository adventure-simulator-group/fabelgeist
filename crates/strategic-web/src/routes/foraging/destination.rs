//! Typed forage completion destinations, encoded only for presentation.

use super::{notice::ForageFeedback, request::ForageReceiptReference};
use crate::routes::return_url::LocalReturnUrl;
use axum::response::Redirect;

pub(super) struct ForageDialogDestination(String);

impl ForageDialogDestination {
    pub(super) fn receipt(
        return_to: LocalReturnUrl<'_>,
        reference: &ForageReceiptReference,
    ) -> Self {
        Self(format!(
            "{return_to}{}forage=true&forage_receipt={reference}",
            return_to.query_separator()
        ))
    }

    pub(super) fn failure(return_to: LocalReturnUrl<'_>, code: ForageFeedback) -> Self {
        Self(format!(
            "{return_to}{}forage=true&forage_error={code}",
            return_to.query_separator()
        ))
    }

    pub(super) fn redirect(self) -> Redirect {
        Redirect::to(&self.0)
    }
}

impl std::fmt::Display for ForageDialogDestination {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
