//! Private investigation authority and observer-safe gateway projections.

//! Implementation is partitioned by behavior domain below. The fragments
//! intentionally share this module scope because SpacetimeDB discovers table,
//! reducer, and view macros in this module and generates accessor names from
//! that scope. Moving those declarations into ordinary child modules changes
//! generated bindings. Macro-free calculations remain grouped with the
//! authority or observer-safe projection that owns them; tests are partitioned
//! by evidence, projection, site/action, and authority behavior.

use adventuresim_world_schema::calendar::StrategicMinute;

#[cfg(test)]
pub(crate) const INVESTIGATION_SOURCE: &str = concat!(
    include_str!("model.rs"),
    include_str!("geometry.rs"),
    include_str!("projections.rs"),
    include_str!("projections/contact.rs"),
    include_str!("projections/site_context.rs"),
    include_str!("capabilities.rs"),
    include_str!("actions/position.rs"),
    include_str!("actions/live_prerequisites.rs"),
    include_str!("actions.rs"),
    include_str!("actions/execution.rs"),
    include_str!("actions/execution_error.rs"),
    include_str!("actions/admission_error.rs"),
    include_str!("actions/result_provenance.rs"),
    include_str!("actions/route_admission.rs"),
    include_str!("sites.rs"),
    include_str!("sites/provenance.rs"),
    include_str!("claims.rs"),
);

#[cfg(feature = "authority-tests")]
mod authority_tests;

#[path = "actions/result_provenance.rs"]
mod result_provenance;
#[path = "actions/route_admission.rs"]
mod route_admission;
use action::{InvestigationActionKind, Terrain};
#[path = "actions/admission_error.rs"]
mod admission_error;
use admission_error::InvestigationAdmissionError;
#[path = "actions/execution.rs"]
mod execution;
pub(crate) use execution::perform_investigation_action_authorized;
#[path = "actions/execution_error.rs"]
mod execution_error;
pub(crate) use execution_error::InvestigationExecutionError;
#[path = "actions/position.rs"]
mod action_position;
use action_position::validate_action_position;
#[path = "actions/live_prerequisites.rs"]
mod live_prerequisites;
use live_prerequisites::validate_live_action_prerequisites;

mod geometry;
use geometry::coordinate_area_contains_e7;

include!("model.rs");
include!("projections.rs");
include!("capabilities.rs");
include!("actions.rs");
include!("sites.rs");
include!("claims.rs");
include!("tests.rs");
