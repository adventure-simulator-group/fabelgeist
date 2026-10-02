//! Trigger guards, barrel bands, sights and stock decoration.
use super::*;
impl Assembly<'_> {
    pub(super) fn trigger_and_ramrod(&mut self, p: &FirearmParameters) -> Result<(), String> {
        let detail = self.detail;
        let start = p.length.get() - p.barrel_length.get();
        let furniture = p
            .furniture_material
            .or(p.lock_material)
            .unwrap_or(Material::Steel);
        let depth = p.stock_depth.get();
        let lock = p.lock_position.get();
        let front = lock - 0.018;
        let rear = lock - p.trigger_length.get() * 1.05;
        let top = -depth * 0.48;
        let bottom = -depth * 0.92;
        self.sweep(
            &[
                [0.0, front, top],
                [0.0, front - 0.008, bottom],
                [0.0, rear, bottom],
                [0.0, rear - 0.008, top],
            ],
            Section::Round,
            0.007,
            0.007,
            furniture,
            "trigger guard in side elevation",
        )?;
        self.add(
            guards::tube(
                &[[0.0, start + 0.02], [0.0, p.length.get() - 0.035]],
                p.ramrod_radius.get(),
                8,
                detail,
            )?
            .transform([0.0; 3], [0.0, 0.0, -depth * 0.43]),
            if p.stock_style == FirearmStockStyle::Shoulder {
                Material::RedBeech
            } else {
                Material::Cherry
            },
            "ramrod",
        );

        Ok(())
    }
    pub(super) fn bands(&mut self, p: &FirearmParameters, centers: &[f64]) -> Result<(), String> {
        let start = p.length.get() - p.barrel_length.get();
        let outer = p.bore.get() / 2.0 + p.barrel_wall.get();
        let furniture = p
            .furniture_material
            .or(p.lock_material)
            .unwrap_or(Material::Steel);
        let depth = p.stock_depth.get();
        let highest = centers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        for index in 0..p.band_count.0 {
            let y = start
                + p.barrel_length.get()
                    * (0.20
                        + index as f64 * 0.62 / (p.band_count.0.saturating_sub(1).max(1)) as f64);
            let half = p.fore_width.get() * 0.58;
            let bottom = -depth * 0.44;
            let top = highest + outer * 1.18;
            self.sweep(
                &[
                    [-half, y, bottom],
                    [-half, y, top],
                    [half, y, top],
                    [half, y, bottom],
                    [-half, y, bottom],
                ],
                Section::Round,
                0.003,
                0.003,
                furniture,
                &format!("barrel band {} XZ enclosure", index + 1),
            )?;
        }

        Ok(())
    }
    pub(super) fn sights(&mut self, p: &FirearmParameters, centers: &[f64]) -> Result<(), String> {
        let detail = self.detail;
        let start = p.length.get() - p.barrel_length.get();
        let outer = p.bore.get() / 2.0 + p.barrel_wall.get();
        let highest = centers.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        match p.sight_style {
            FirearmSightStyle::None => {}
            FirearmSightStyle::Bead => {
                self.add(
                    Solid::lathe(&[[0.0, 0.003], [0.012, 0.001]], 8, 1.0, false, detail)?
                        .transform([0.0; 3], [0.0, p.length.get() - 0.045, highest + outer]),
                    Material::Steel,
                    "front bead sight",
                );
            }
            FirearmSightStyle::Notch => {
                self.add(
                    Solid::prism(
                        &[
                            [-0.008, 0.0],
                            [-0.003, 0.010],
                            [0.0, 0.006],
                            [0.003, 0.010],
                            [0.008, 0.0],
                        ],
                        0.003,
                        detail,
                    )?
                    .transform([0.0; 3], [0.0, start + 0.06, highest + outer]),
                    Material::Steel,
                    "rear notch sight",
                );
            }
        }

        Ok(())
    }
    pub(super) fn facing(&mut self, p: &FirearmParameters) -> Result<(), String> {
        let detail = self.detail;
        let stations = stations(p);
        let start = p.length.get() - p.barrel_length.get();
        let depth = p.stock_depth.get();
        if p.facing_style == FirearmFacingStyle::Horn {
            let plaque = Solid::prism(
                &[
                    [-0.030, -0.018],
                    [0.028, -0.012],
                    [0.034, 0.010],
                    [-0.020, 0.016],
                ],
                p.facing_thickness.get(),
                detail,
            )?;
            for side in [-1.0, 1.0] {
                self.add(
                    plaque.clone().transform(
                        [0.0, side * 90.0, 0.0],
                        [
                            side * (p.waist_width.get() / 2.0 + p.facing_thickness.get() / 2.0),
                            start * 0.62,
                            -depth * 0.35,
                        ],
                    ),
                    p.facing_material.unwrap_or(Material::Horn),
                    "shaped stock facing plaque",
                );
            }
            for index in 0..3 {
                let y = start * (0.35 + index as f64 * 0.14);
                let station = StockStation::at(&stations, y);
                self.add(
                    Solid::lathe(&[[-0.001, 0.005], [0.001, 0.005]], 12, 1.0, false, detail)?
                        .transform(
                            [0.0, 0.0, 90.0],
                            [
                                station.width / 2.0,
                                y,
                                station.bottom + (station.top - station.bottom) * 0.55,
                            ],
                        ),
                    p.inlay_material.unwrap_or(Material::MotherOfPearl),
                    &format!("stock inlay rosette {}", index + 1),
                );
            }
        }

        Ok(())
    }
}
