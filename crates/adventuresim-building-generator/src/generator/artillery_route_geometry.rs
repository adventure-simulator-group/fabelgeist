//! Identity lookup and nearest-curtain choice for the authored artillery route.
use super::*;

pub(super) fn position(
    nodes: &[crate::ArtilleryRouteNode],
    node: crate::ArtilleryRouteNodeId,
) -> Result<Vec3, GenerationError> {
    nodes
        .iter()
        .find(|candidate| candidate.id == node)
        .map(|candidate| candidate.position)
        .ok_or(GenerationError::MissingArtilleryRouteNode { node })
}

pub(super) fn nearest_curtain(
    rondel: &crate::ArtilleryRondelAssembly,
    curtain_nodes: &[crate::ArtilleryRouteNodeId],
    nodes: &[crate::ArtilleryRouteNode],
) -> Result<usize, GenerationError> {
    let curtain_position = |curtain: crate::ArtilleryCurtainId| {
        let index = curtain.0 as usize;
        let node = curtain_nodes
            .get(index)
            .ok_or(GenerationError::MissingArtilleryCurtain { curtain })?;
        position(nodes, *node)
    };
    let [left, right] = rondel.adjoining_curtains;
    let lp = curtain_position(left)?;
    let rp = curtain_position(right)?;
    let centre = rondel.anchor.metres();
    let left_distance = Vec2::new(lp.x, lp.z).distance(centre);
    let right_distance = Vec2::new(rp.x, rp.z).distance(centre);
    // Iterator::min_by selects the later represented value on equal distances.
    Ok(if left_distance.total_cmp(&right_distance).is_lt() {
        left.0 as usize
    } else {
        right.0 as usize
    })
}
