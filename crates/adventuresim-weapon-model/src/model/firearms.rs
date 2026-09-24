//! Stock, barrel and furniture construction for matchlock and wheellock arms.
use super::*;
mod furniture;
mod mechanisms;

pub(super) fn stations(p: &FirearmParameters) -> Vec<StockStation> {
    let start = p.length.get() - p.barrel_length.get();
    let butt = p.butt_width.get();
    let waist = p.waist_width.get();
    let fore = p.fore_width.get();
    let depth = p.stock_depth.get();
    let drop = p.butt_drop.get();
    let values = if p.stock_style == FirearmStockStyle::Pistol {
        vec![
            [0.0, butt * 0.72, -depth - drop, -drop * 0.90],
            [
                start * 0.12,
                butt,
                -depth * 0.96 - drop * 0.72,
                -drop * 0.65,
            ],
            [
                start * 0.32,
                butt * 0.92,
                -depth * 0.82 - drop * 0.38,
                -drop * 0.35,
            ],
            [start * 0.56, waist * 1.10, -depth * 0.69, -drop * 0.12],
            [start * 0.78, waist, -depth * 0.60, -0.002],
            [start, waist * 0.94, -depth * 0.52, 0.0],
            [
                start + (p.length.get() - start) * 0.48,
                fore * 1.08,
                -depth * 0.44,
                0.0,
            ],
            [p.length.get() - 0.025, fore, -depth * 0.38, 0.0],
        ]
    } else {
        vec![
            [0.0, butt, -depth - drop, -drop],
            [
                start * 0.10,
                butt * 1.05,
                -depth * 1.02 - drop * 0.78,
                -drop * 0.92,
            ],
            [
                start * 0.28,
                butt * 0.98,
                -depth - drop * 0.55,
                -drop * 0.72,
            ],
            [
                start * 0.48,
                butt * 0.88,
                -depth * 0.96 - drop * 0.22,
                -drop * 0.45,
            ],
            [start * 0.68, butt * 0.72, -depth * 0.88, -drop * 0.22],
            [start * 0.84, waist * 1.25, -depth * 0.78, -drop * 0.08],
            [start, waist, -depth * 0.65, 0.0],
            [
                start + (p.length.get() - start) * 0.52,
                fore * 1.06,
                -depth * 0.47,
                0.0,
            ],
            [p.length.get() - 0.045, fore, -depth * 0.40, 0.0],
        ]
    };
    values
        .into_iter()
        .map(|[y, width, bottom, top]| StockStation::new(y, width, bottom, top))
        .collect()
}

pub(super) struct Assembly<'a> {
    pub(super) parts: Vec<PartSource>,
    pub(super) resolved: &'a ResolvedComponent,
    pub(super) detail: Detail,
}
impl Assembly<'_> {
    pub(super) fn add(&mut self, solid: Solid, material: Material, label: &str) -> &mut PartSource {
        self.parts
            .push(PartSource::new(solid, material, label, &self.resolved.id));
        self.parts.last_mut().unwrap()
    }
    pub(super) fn cuboid(
        &mut self,
        size: Point,
        offset: Point,
        material: Material,
        label: &str,
    ) -> Result<&mut PartSource, String> {
        Ok(self.add(
            Solid::cuboid(size, self.detail)?.transform([0.0; 3], offset),
            material,
            label,
        ))
    }
    pub(super) fn sweep(
        &mut self,
        points: &[Point],
        section: Section,
        width: f64,
        depth: f64,
        material: Material,
        label: &str,
    ) -> Result<&mut PartSource, String> {
        Ok(self.add(
            Solid::sweep(
                points,
                &Sweep {
                    section,
                    width,
                    depth,
                    ..Sweep::default()
                },
                self.detail,
            )?,
            material,
            label,
        ))
    }
}

pub(super) fn firearm(
    r: &ResolvedComponent,
    p: &FirearmParameters,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    let stations = stations(p);
    let start = p.length.get() - p.barrel_length.get();
    let outer = p.bore.get() / 2.0 + p.barrel_wall.get();
    let mut assembly = Assembly {
        parts: Vec::new(),
        resolved: r,
        detail,
    };
    assembly.add(
        Solid::stock(&stations)?,
        r.component.material.unwrap_or(Material::Wood),
        "combined-profile firearm stock",
    );
    let centers = if p.barrel_count.0 == 2 {
        vec![outer * 3.15, outer * 1.05]
    } else {
        vec![outer]
    };
    assembly.stock_pommel(p)?;
    assembly.barrels(p, &centers)?;
    firelocks::locks(&mut assembly, p, &centers, outer, start)?;
    assembly.trigger_and_ramrod(p)?;
    assembly.bands(p, &centers)?;
    assembly.sights(p, &centers)?;
    assembly.facing(p)?;
    Ok(assembly.parts)
}
