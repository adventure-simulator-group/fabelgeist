//! Admit a complete player-safe receipt before presentation or acknowledgement.

use super::request::{ForageReceiptReference, ForageReceiptReferenceError};
use crate::spacetimedb::BackendForageReceipt;
use adventuresim_core::{
    foraging::{
        EmptyForageYield, ForageLegalOutcomeError, ForagePublicLegalOutcome, ForageResource,
        ForageYieldQuantity, UnknownForageResource,
    },
    identity::CharacterId,
};
use adventuresim_world_schema::calendar::{MINUTES_PER_HOUR, StrategicDuration};
use maud::{Markup, html};

#[derive(Debug)]
pub(super) struct ForageReceipt {
    completion: ForageCompletion,
    legal_outcome: ForagePublicLegalOutcome,
}

#[derive(Debug)]
enum ForageCompletion {
    Interrupted(StrategicDuration),
    Completed(Vec<ForageReceiptYield>),
}

#[derive(Debug)]
struct ForageReceiptYield {
    resource: ForageResource,
    quantity: ForageYieldQuantity,
}

impl ForageReceipt {
    pub(super) fn admit(
        native: BackendForageReceipt,
        actor: CharacterId,
        reference: &ForageReceiptReference,
    ) -> std::result::Result<Self, ForageReceiptError> {
        let legal_outcome = ForagePublicLegalOutcome::try_from(native.legal_outcome.as_str())?;
        let actual_actor = CharacterId::from(native.character_id);
        if actual_actor != actor {
            return Err(ForageReceiptError::Actor {
                expected: actor,
                actual: actual_actor,
            });
        }
        let actual_reference = ForageReceiptReference::try_from(native.request_id.as_str())?;
        if actual_reference != *reference {
            return Err(ForageReceiptError::Reference {
                expected: reference.clone(),
                actual: actual_reference,
            });
        }
        if native.yielded_item_ids.len() != native.yielded_quantities.len() {
            return Err(ForageReceiptError::YieldColumns {
                resources: ForageYieldCount(native.yielded_item_ids.len()),
                quantities: ForageYieldCount(native.yielded_quantities.len()),
            });
        }
        let completion = if native.interrupted {
            if !native.yielded_item_ids.is_empty() {
                return Err(ForageReceiptError::InterruptedHarvest);
            }
            ForageCompletion::Interrupted(StrategicDuration::new(native.elapsed_minutes))
        } else {
            let mut harvest: Vec<ForageReceiptYield> = Vec::new();
            for (item, quantity) in native
                .yielded_item_ids
                .iter()
                .zip(native.yielded_quantities)
            {
                let resource = ForageResource::try_from(item.as_str())?;
                let quantity = ForageYieldQuantity::try_from(quantity).map_err(
                    |source: EmptyForageYield| -> ForageReceiptError {
                        ForageReceiptError::EmptyYield { resource, source }
                    },
                )?;
                for previous in &harvest {
                    if previous.resource == resource {
                        return Err(ForageReceiptError::DuplicateYield(resource));
                    }
                }
                harvest.push(ForageReceiptYield { resource, quantity });
            }
            ForageCompletion::Completed(harvest)
        };
        Ok(Self {
            completion,
            legal_outcome,
        })
    }

    pub(super) fn render(&self) -> Markup {
        html! {
            div role="status" aria-live="polite" {
                @match &self.completion {
                    ForageCompletion::Interrupted(elapsed) => {
                        p { "The search was interrupted after " (elapsed.get() / MINUTES_PER_HOUR) " hour(s). Nothing was gathered." }
                    }
                    ForageCompletion::Completed(harvest) => {
                        @if harvest.is_empty() {
                            p { "The search found nothing." }
                        } @else {
                            h3 { "Gathered" }
                            ul {
                                @for found in harvest {
                                    li { (found.quantity) " × " (found.resource.name) }
                                }
                            }
                        }
                    }
                }
                @if self.legal_outcome == ForagePublicLegalOutcome::Unnoticed {
                    p { "The illegal search went unnoticed." }
                } @else if self.legal_outcome == ForagePublicLegalOutcome::Noticed {
                    p { "The illegal search was noticed. Local Infamy increased." }
                }
            }
        }
    }
}

/// Cardinality of one native receipt column, retained for shape diagnostics.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct ForageYieldCount(usize);

impl std::fmt::Display for ForageYieldCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum ForageReceiptError {
    #[error("{0}")]
    LegalOutcome(#[from] ForageLegalOutcomeError),
    #[error("{0}")]
    InvalidReference(#[from] ForageReceiptReferenceError),
    #[error("Forage receipt actor {actual} differs from requested actor {expected}")]
    Actor {
        expected: CharacterId,
        actual: CharacterId,
    },
    #[error("Forage receipt reference {actual} differs from requested reference {expected}")]
    Reference {
        expected: ForageReceiptReference,
        actual: ForageReceiptReference,
    },
    #[error("Forage receipt has {resources} resource keys and {quantities} quantities")]
    YieldColumns {
        resources: ForageYieldCount,
        quantities: ForageYieldCount,
    },
    #[error("An interrupted forage receipt cannot contain a harvest")]
    InterruptedHarvest,
    #[error("{0}")]
    UnknownResource(#[from] UnknownForageResource),
    #[error("Forage receipt contains a zero yield for {}", .resource.item_id)]
    EmptyYield {
        resource: ForageResource,
        #[source]
        source: EmptyForageYield,
    },
    #[error("Forage receipt repeats resource {}", .0.item_id)]
    DuplicateYield(ForageResource),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn reference() -> ForageReceiptReference {
        ForageReceiptReference::try_from("a".repeat(64).as_str()).unwrap()
    }

    fn native_receipt() -> BackendForageReceipt {
        BackendForageReceipt {
            character_id: 7,
            request_id: reference().to_string(),
            elapsed_minutes: 90,
            yielded_item_ids: vec!["sage".into(), "wild_berries".into()],
            yielded_quantities: vec![1, u16::MAX],
            interrupted: false,
            legal_outcome: "legal".into(),
        }
    }

    #[test]
    fn admitted_harvest_preserves_order_catalog_labels_and_full_quantity_range() {
        let receipt = ForageReceipt::admit(native_receipt(), 7.into(), &reference()).unwrap();
        let rendered = receipt.render().into_string();
        assert_eq!(
            rendered,
            "<div role=\"status\" aria-live=\"polite\"><h3>Gathered</h3><ul><li>1 × Sage</li><li>65535 × Wild berries</li></ul></div>"
        );
    }

    #[test]
    fn admission_rejects_both_column_mismatches_before_pairing() {
        let mut native = native_receipt();
        native.yielded_quantities.pop();
        assert!(matches!(
            ForageReceipt::admit(native, 7.into(), &reference()),
            Err(ForageReceiptError::YieldColumns {
                resources: ForageYieldCount(2),
                quantities: ForageYieldCount(1),
            })
        ));
        let mut native = native_receipt();
        native.yielded_item_ids.clear();
        assert!(matches!(
            ForageReceipt::admit(native, 7.into(), &reference()),
            Err(ForageReceiptError::YieldColumns {
                resources: ForageYieldCount(0),
                quantities: ForageYieldCount(2),
            })
        ));
    }

    #[test]
    fn admission_classifies_unknown_empty_and_duplicate_yields() {
        let mut native = native_receipt();
        native.yielded_item_ids[0] = "secret-unknown-item".into();
        let error = ForageReceipt::admit(native, 7.into(), &reference()).unwrap_err();
        assert!(matches!(&error, ForageReceiptError::UnknownResource(_)));
        assert!(error.source().unwrap().is::<UnknownForageResource>());
        assert!(error.to_string().contains("secret-unknown-item"));

        let mut native = native_receipt();
        native.yielded_quantities[0] = 0;
        let Err(ForageReceiptError::EmptyYield { resource, source }) =
            ForageReceipt::admit(native, 7.into(), &reference())
        else {
            panic!("zero quantity must retain its resource");
        };
        assert_eq!(resource, ForageResource::try_from("sage").unwrap());
        assert_eq!(source, EmptyForageYield);

        let mut native = native_receipt();
        native.yielded_item_ids[1] = "sage".into();
        assert!(matches!(
            ForageReceipt::admit(native, 7.into(), &reference()),
            Err(ForageReceiptError::DuplicateYield(_))
        ));
    }

    #[test]
    fn interruption_cannot_carry_a_harvest() {
        let mut native = native_receipt();
        native.interrupted = true;
        assert!(matches!(
            ForageReceipt::admit(native, 7.into(), &reference()),
            Err(ForageReceiptError::InterruptedHarvest)
        ));
    }

    #[test]
    fn admission_binds_the_actual_receipt_to_the_selected_actor_and_reference() {
        let Err(ForageReceiptError::Actor { expected, actual }) =
            ForageReceipt::admit(native_receipt(), 8.into(), &reference())
        else {
            panic!("a different actor must not be admitted");
        };
        assert_eq!(expected, CharacterId::from(8));
        assert_eq!(actual, CharacterId::from(7));

        let mut native = native_receipt();
        native.request_id = "b".repeat(64);
        let Err(ForageReceiptError::Reference { expected, actual }) =
            ForageReceipt::admit(native, 7.into(), &reference())
        else {
            panic!("a different request must not be admitted");
        };
        assert_eq!(expected, reference());
        assert_eq!(actual.to_string(), "b".repeat(64));

        let mut native = native_receipt();
        native.request_id = "not-hexadecimal".into();
        let error = ForageReceipt::admit(native, 7.into(), &reference()).unwrap_err();
        assert!(matches!(&error, ForageReceiptError::InvalidReference(_)));
        assert!(error.source().unwrap().is::<ForageReceiptReferenceError>());
    }
    #[test]
    fn receipt_rendering_preserves_notices_and_rejects_unknown_outcomes() {
        let reference = reference();
        let mut receipt = BackendForageReceipt {
            character_id: 7,
            request_id: "a".repeat(64),
            elapsed_minutes: 90,
            yielded_item_ids: Vec::new(),
            yielded_quantities: Vec::new(),
            interrupted: false,
            legal_outcome: String::new(),
        };
        for (outcome, notice) in [
            (ForagePublicLegalOutcome::Legal, None),
            (
                ForagePublicLegalOutcome::Unnoticed,
                Some("The illegal search went unnoticed."),
            ),
            (
                ForagePublicLegalOutcome::Noticed,
                Some("The illegal search was noticed. Local Infamy increased."),
            ),
        ] {
            receipt.legal_outcome = outcome.to_string();
            let rendered = ForageReceipt::admit(receipt.clone(), 7.into(), &reference)
                .unwrap()
                .render()
                .into_string();
            assert!(rendered.contains("The search found nothing."));
            match notice {
                Some(notice) => assert!(rendered.contains(notice)),
                None => assert!(!rendered.contains("illegal search")),
            }
        }
        receipt.interrupted = true;
        let rendered = ForageReceipt::admit(receipt.clone(), 7.into(), &reference)
            .unwrap()
            .render()
            .into_string();
        assert!(
            rendered.contains("The search was interrupted after 1 hour(s). Nothing was gathered.")
        );
        receipt.legal_outcome = "lawful".into();
        assert!(ForageReceipt::admit(receipt, 7.into(), &reference).is_err());
    }
}
