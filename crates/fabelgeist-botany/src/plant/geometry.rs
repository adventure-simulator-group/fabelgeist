//! Secondary growth: pipe-model diameters and retained heartwood.
use super::*;

impl Plant {
    pub(super) fn update_diameters(&mut self, species: &PlantType, prototypes: &PrototypeLibrary) {
        // Diameters, basipetally: Eq. 8, the pipe model with n = 2, for the
        // sapwood. Wood is never taken away: sapwood that a shed branch drew
        // on stops conducting but stays, as heartwood.
        for u in (0..self.modules.len()).rev() {
            let module = &self.modules[u];
            let prototype = prototypes.get(module.prototype);
            let Some(offset) = module.node_offset else {
                continue;
            };
            let offset = offset.index();
            for n in (0..prototype.node_count()).rev() {
                let mut sum = 0.0f32;
                let mut thickest = 0.0f32;
                let mut continuation = |node: &Node| {
                    sum += node.sapwood * node.sapwood;
                    thickest = thickest.max(node.diameter);
                };
                for &c in &prototype.children[n] {
                    continuation(&self.nodes[offset + c as usize]);
                }
                let slot = prototype.terminal_slot[n];
                if slot != NONE {
                    let child = module.children[slot as usize];
                    if let Some(child) = child
                        && let Some(offset) = self.modules[child.index()].node_offset
                    {
                        continuation(&self.nodes[offset.index()]);
                    }
                }
                let node = &mut self.nodes[offset + n];
                // A new shoot starts thin and reaches its tip's diameter over
                // its first stretch of growth, so that what it adds below it
                // comes on gradually too; and a grown segment is never thinner
                // than a tip, or a shoot budding from its end, starting thin,
                // would thin everything below it.
                let tip = if n > 0 && node.age.stage() > 0.0 {
                    species.tip_diameter * (node.age.stage() / 0.2).min(1.0)
                } else {
                    0.0
                };
                let (sapwood, end) = if sum.sqrt() > tip {
                    (sum.sqrt(), thickest)
                } else {
                    (tip, tip * 0.5)
                };
                let laid = node.diameter;
                node.sapwood = sapwood;
                node.heartwood = node.heartwood.max(laid * laid - sapwood * sapwood);
                node.diameter = (sapwood * sapwood + node.heartwood).sqrt();
                node.end_diameter = end;
            }
        }
    }
}
