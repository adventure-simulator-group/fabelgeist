//! Resolve specialist envelopes and their final timber and roof attachments.
use super::*;

impl BuildingPlan {
    pub(super) fn resolve_architectural_envelope(
        mut self,
        program: &BuildingProgram,
        edits: &[BuildingEdit],
    ) -> Result<Self, GenerationError> {
        self.workplace = crate::workplace::resolve_workplace(
            program,
            &mut self.wall_assemblies,
            &mut self.resolved_geometry,
        )?;
        self.small_church = small_church::resolve(
            program,
            &mut self.wall_assemblies,
            &mut self.resolved_geometry,
        )?;
        self.church = urban_church::resolve(
            program,
            &self.square_towers,
            &mut self.wall_assemblies,
            &mut self.opening_assemblies,
            &mut self.stairs,
            &mut self.resolved_geometry,
        )?;
        fortified_envelope::resolve(
            program,
            &self.towers,
            &self.crowns,
            &self.projected_defenses,
            &mut self.wall_assemblies,
            &mut self.opening_assemblies,
            &mut self.resolved_geometry,
        )?;
        self.artillery_castle = resolve_artillery_castle(
            program,
            &self.towers,
            &mut self.wall_assemblies,
            &mut self.opening_assemblies,
            &mut self.resolved_geometry,
        )?;

        let (roof_assemblies, timber_frame) = framed_roofs::resolve(
            program,
            edits,
            &self.roofs,
            &self.roof_dormers,
            &self.towers,
            &self.square_towers,
            &mut self.stairs,
            &mut self.wall_assemblies,
            &mut self.opening_assemblies,
            &mut self.resolved_geometry,
        )?;
        self.roof_assemblies = roof_assemblies;
        self.timber_frame = timber_frame;
        // Corner bonds must be resolved against the final timber-infill depth,
        // after the semantic frame has replaced the exterior structural layer.
        wall_corner_bonds::resolve(&self.wall_assemblies, &mut self.resolved_geometry)?;
        if let Some(church) = &mut self.church {
            church.roof_assemblies = self.roof_assemblies.iter().map(|roof| roof.id).collect();
        }

        church_ground::resolve(crate::spiral_stairs::resolve(self)?)
    }
}
