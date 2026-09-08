//! Resolved solids without renderer normals or shading buffers.
use super::*;

pub(crate) struct ConstructedPart<'a> {
    pub component: &'a ComponentDesign,
    pub mesh: RawMesh,
    pub origin: [f32; 3],
}

pub(crate) struct ConstructedWeapon<'a> {
    pub parts: Vec<ConstructedPart<'a>>,
    pub grip: [f32; 3],
}

impl<'a> ConstructedWeapon<'a> {
    pub fn new(design: &'a WeaponDesign) -> Result<Self, Vec<ValidationError>> {
        crate::validate(design)?;
        let by_id = design
            .components
            .iter()
            .map(|component| (component.id.as_str(), component))
            .collect();
        let mut origins = HashMap::new();
        let mut parts = Vec::with_capacity(design.components.len());
        let mut grip = [0.0; 3];
        for component in &design.components {
            let origin = resolve_origin(component, &by_id, &mut origins, &mut Vec::new())
                .map_err(|_| vec![ValidationError::AttachmentCycle(component.id.clone())])?;
            let mut mesh = RawMesh::from_shape(&component.shape);
            mesh.translate(origin);
            if component.role == ComponentRole::Grip {
                grip = origin;
                grip[1] += component.shape.axial_length().meters() * 0.5;
            }
            parts.push(ConstructedPart {
                component,
                mesh,
                origin,
            });
        }
        Ok(Self { parts, grip })
    }
}
