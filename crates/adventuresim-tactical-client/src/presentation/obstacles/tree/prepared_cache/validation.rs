//! Validate the complete fixed-Full dependency set before live asset mutation.
use super::*;
impl Payload {
    pub(super) fn validate(
        &self,
        input: &TacticalSceneInput,
        environment: &SceneEnvironment,
        id: PlantId,
    ) -> Result<()> {
        let specimen = SourceSpecimen::find(id)?;
        ensure!(
            self.binding.source == id.word()
                && self.binding.seed == specimen.seed
                && self.binding.species == specimen.species,
            "tree cache selected source binding"
        );
        ensure!(
            self.binding.input == input.digest()?.as_bytes()
                && self.binding.environment == serde_json::to_vec(environment)?,
            "tree cache input/environment binding"
        );
        ensure!(
            self.binding.cache_key
                == playable_cache_key(specimen.species, specimen.seed, environment),
            "tree cache site identity"
        );
        self.source.validate()?;
        self.detailed_trunk
            .validate_part(mesh::Part::DetailedTrunk)?;
        self.mid_trunk.validate_part(mesh::Part::MidTrunk)?;
        ensure!(
            self.detailed_trunk.vertices() > 0 && self.mid_trunk.vertices() > 0,
            "required tree trunks"
        );
        self.card
            .validate_full_dependency(specimen.species, specimen.seed)?;
        let bake = self.card.runtime();
        mesh::PreparedMesh::from_mesh(&bake.mesh)?;
        ensure!(
            self.groups.len() == bake.clusters.len(),
            "tree cache group count"
        );
        for (group, cluster) in self.groups.iter().zip(&bake.clusters) {
            ensure!(
                group.primary_group == cluster.primary_group
                    && group.center == cluster.center.to_array()
                    && group.radius == cluster.radius,
                "tree cache group layout binding"
            );
            group.wood.validate_part(mesh::Part::Wood)?;
            group.buds.validate_part(mesh::Part::Buds)?;
            group.leaves.validate_part(mesh::Part::Leaves)?;
            ensure!(
                group.wood.vertices() > 0 && group.leaves.vertices() > 0,
                "required tree crown mesh"
            );
        }
        Ok(())
    }
}
