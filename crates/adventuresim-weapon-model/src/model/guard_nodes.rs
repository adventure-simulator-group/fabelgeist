//! Evaluate guard node bindings in dependency order in component coordinates.
use super::*;
use std::collections::BTreeMap;
pub(super) fn resolve(
    p: &mut GuardAssemblyParameters,
    frames: &BTreeMap<String, Point>,
    offset: Point,
    rotation: Point,
) -> Result<(), String> {
    let order = p.binding_order()?;
    let Some(bindings) = &p.node_bindings else {
        return Ok(());
    };
    for name in order {
        if !p.nodes.contains_key(&name) {
            return Err("binding references missing guard node".into());
        }
        let point = match &bindings[&name] {
            NodeBinding::Frame {
                frame,
                offset: delta,
            } => {
                let target = *frames
                    .get(frame)
                    .ok_or_else(|| format!("missing bound node frame {frame}"))?;
                add(
                    inverse_rotate(sub(target, offset), rotation),
                    delta.map_or([0.0; 3], |p| p.map(Metres::get)),
                )
            }
            NodeBinding::Between {
                between,
                t,
                offset: delta,
            } => {
                let a = p
                    .nodes
                    .get(&between[0])
                    .ok_or("missing interpolation node")?
                    .map(Metres::get);
                let b = p
                    .nodes
                    .get(&between[1])
                    .ok_or("missing interpolation node")?
                    .map(Metres::get);
                add(
                    lerp(a, b, t.map_or(0.5, Ratio::get)),
                    delta.map_or([0.0; 3], |p| p.map(Metres::get)),
                )
            }
        };
        p.nodes.insert(
            name,
            [
                Metres::new(point[0])?,
                Metres::new(point[1])?,
                Metres::new(point[2])?,
            ],
        );
    }
    Ok(())
}
