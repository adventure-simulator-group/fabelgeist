//! Strategic operation failures retain leaf causes until the reducer Display port.

use adventuresim_core::{
    autoresolve::{AuthoredThreatError, BattleResolutionError},
    elapsed::ElapsedError,
    physical_object::CustodyIdentityError,
};
use fabelgeist_geometry::GeometryError;

pub type StrategicOperationResult<T> = Result<T, StrategicOperationError>;

/// A leaf failure and its outer settlement-activity contexts.
#[derive(Debug)]
pub struct StrategicOperationError {
    cause: StrategicOperationCause,
    settlement_contexts: Vec<SettlementActivityContext>,
}

#[derive(Debug)]
struct SettlementActivityContext {
    settlement_id: String,
    stage: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum StrategicOperationCause {
    #[error(transparent)]
    EquipmentLaw(#[from] crate::equipment_law::EquipmentLawError),
    #[error(transparent)]
    AuthoredThreat(#[from] AuthoredThreatError),
    #[error(transparent)]
    Resolution(#[from] BattleResolutionError),
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error(transparent)]
    Timing(#[from] ElapsedError),
    #[error(transparent)]
    Custody(#[from] CustodyIdentityError),
    #[error("Autopsy demo autoresolve could not produce its required death")]
    RequiredDeathUnavailable,
    #[error("equipped weapon {item} is absent from the authored catalog")]
    MissingWeaponDefinition { item: String },
    #[error("equipped weapon {item} has no authored equipment geometry")]
    MissingWeaponGeometry { item: String },
    #[error("parametric weapon {item} has no valid physical recipe")]
    MissingWeaponRecipe { item: String },
    #[error("parametric weapon {item} has invalid combat geometry")]
    InvalidWeaponCombatGeometry { item: String },
    #[error("{0}")]
    Operation(String),
}
impl StrategicOperationError {
    pub(crate) fn cause(&self) -> &StrategicOperationCause {
        &self.cause
    }

    pub(crate) fn in_settlement_activity(
        mut self,
        settlement_id: &str,
        stage: &'static str,
    ) -> Self {
        self.settlement_contexts.push(SettlementActivityContext {
            settlement_id: settlement_id.to_owned(),
            stage,
        });
        self
    }
}

impl std::fmt::Display for StrategicOperationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for context in self.settlement_contexts.iter().rev() {
            write!(
                formatter,
                "Settlement activity for {} failed during {}: ",
                context.settlement_id, context.stage
            )?;
        }
        std::fmt::Display::fmt(&self.cause, formatter)
    }
}

impl std::error::Error for StrategicOperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

impl From<StrategicOperationCause> for StrategicOperationError {
    fn from(cause: StrategicOperationCause) -> Self {
        Self {
            cause,
            settlement_contexts: Vec::new(),
        }
    }
}

macro_rules! operation_cause_from {
    ($($source:ty => $variant:ident),+ $(,)?) => {
        $(impl From<$source> for StrategicOperationError {
            fn from(source: $source) -> Self {
                StrategicOperationCause::$variant(source).into()
            }
        })+
    };
}

operation_cause_from! {
    crate::equipment_law::EquipmentLawError => EquipmentLaw,
    AuthoredThreatError => AuthoredThreat,
    BattleResolutionError => Resolution,
    GeometryError => Geometry,
    ElapsedError => Timing,
    CustodyIdentityError => Custody,
    String => Operation,
}

impl From<&str> for StrategicOperationError {
    fn from(message: &str) -> Self {
        StrategicOperationCause::Operation(message.to_owned()).into()
    }
}
