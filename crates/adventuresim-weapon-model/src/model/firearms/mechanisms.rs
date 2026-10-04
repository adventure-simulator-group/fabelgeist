//! Barrel walls, sealed breeches and stock-end construction.
use super::*;
impl Assembly<'_> {
    pub(super) fn stock_pommel(&mut self, p: &FirearmParameters) -> Result<(), String> {
        let r = self.resolved;
        let detail = self.detail;
        if p.stock_style == FirearmStockStyle::Pistol {
            let pommel = PommelParameters {
                construction: PommelConstruction::Writhen,
                base_construction: None,
                profile: None,
                segments: p.radial_segments,
                width_scale: None,
                length_scale: None,
                height: Some(Metres::new(p.butt_width.get() * 1.12)?),
                diameter: Some(Metres::new(p.butt_width.get() * 1.16)?),
                thickness: Some(Metres::new(p.butt_width.get() * 0.5)?),
                face_convexity: None,
                rim_bevel: None,
                facets: None,
                flute_count: Some(Count(8)),
                flute_depth: Some(Ratio::new(0.10)?),
                twist: Some(Degrees::new(65.0)?),
                outline_style: None,
                notch_depth: None,
                lobe_spread: None,
                shoulder_width: None,
                sockets: None,
                ornaments: None,
            };
            let mut parts = pommels::pommel(r, &pommel, detail)?;
            for part in &mut parts {
                part.solid = std::mem::take(&mut part.solid).transform(
                    [0.0; 3],
                    [0.0, 0.002, -p.stock_depth.get() - p.butt_drop.get()],
                );
                part.label = "solid spiral-fluted bulb pommel".into();
                part.material = r.component.material.unwrap_or(Material::Cherry);
            }
            self.parts.extend(parts);
        }

        Ok(())
    }
    pub(super) fn barrels(&mut self, p: &FirearmParameters, centers: &[f64]) -> Result<(), String> {
        let detail = self.detail;
        let start = p.length.get() - p.barrel_length.get();
        let outer = p.bore.get() / 2.0 + p.barrel_wall.get();
        let material = p.barrel_material.unwrap_or(Material::DarkSteel);
        for (index, &z) in centers.iter().enumerate() {
            let name = if centers.len() == 2 {
                if index == 0 { "upper" } else { "lower" }
            } else {
                "barrel 1"
            };
            let length = if index == 0 {
                p.barrel_length.get()
            } else {
                p.secondary_barrel_length
                    .ok_or("double barrel needs secondary length")?
                    .get()
            };
            let end = start + length;
            let breech = length * p.octagonal_ratio.get();
            let flare = if p.muzzle_style == FirearmMuzzleStyle::Ringed {
                p.muzzle_flare.map_or(0.0, Metres::get)
            } else {
                0.0
            };
            self.add(
                Solid::faceted_socket(
                    &[[start, outer * 1.10], [start + breech, outer]],
                    p.bore.get() / 2.0,
                    8,
                    detail,
                )
                .transform([0.0; 3], [0.0, 0.0, z]),
                material,
                &format!("{name} barrel octagonal breech"),
            );
            self.add(
                Solid::hollow_socket(
                    &[[start + breech, outer], [end, outer + flare]],
                    &[p.bore.get() / 2.0; 2],
                    Some(p.radial_segments.map_or(12, |n| n.0 as usize)),
                    detail,
                )
                .transform([0.0; 3], [0.0, 0.0, z]),
                material,
                &format!("{name} barrel round fore-barrel"),
            );
            self.add(
                Solid::lathe(
                    &[
                        [start - p.barrel_wall.get(), outer * 1.1],
                        [start, outer * 1.1],
                    ],
                    12,
                    1.0,
                    true,
                    detail,
                )?
                .transform([0.0; 3], [0.0, 0.0, z]),
                material,
                &format!("{name} barrel closed breech"),
            );
            if p.muzzle_style == FirearmMuzzleStyle::Ringed {
                self.add(
                    Solid::hollow_socket(
                        &[[end - 0.012, outer + flare], [end, outer + flare]],
                        &[p.bore.get() / 2.0; 2],
                        None,
                        detail,
                    )
                    .transform([0.0; 3], [0.0, 0.0, z]),
                    material,
                    &format!("{name} barrel open muzzle ring"),
                );
            }
        }

        Ok(())
    }
}
