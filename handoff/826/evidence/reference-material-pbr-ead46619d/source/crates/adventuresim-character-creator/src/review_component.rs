//! Native review protocol view of one authored component.

use crate::surface_projection::{LinearSurfaceMaterial, SurfaceProjectionError};
use fabelgeist_armor::{ArmorComponent, ArmorComponentRole, ArmorHinge, PlateMount};
use serde::Serialize;
use std::ops::Range;

/// Borrowed component metadata with checked linear scalar material factors.
/// Geometry ranges and attachment metadata retain their source contracts.
#[derive(Serialize)]
pub struct ReviewComponent<'a> {
    role: ArmorComponentRole,
    vertices: &'a Range<usize>,
    indices: &'a Range<usize>,
    hinge: Option<ArmorHinge>,
    mount: Option<&'a PlateMount>,
    material: Option<LinearSurfaceMaterial>,
}

impl<'a> TryFrom<&'a ArmorComponent> for ReviewComponent<'a> {
    type Error = SurfaceProjectionError;

    fn try_from(component: &'a ArmorComponent) -> Result<Self, Self::Error> {
        let material = component
            .appearance
            .material()
            .map(LinearSurfaceMaterial::try_from)
            .transpose()?;
        Ok(Self {
            role: component.role,
            vertices: &component.vertices,
            indices: &component.indices,
            hinge: component.hinge,
            mount: component.mount.as_ref(),
            material,
        })
    }
}
