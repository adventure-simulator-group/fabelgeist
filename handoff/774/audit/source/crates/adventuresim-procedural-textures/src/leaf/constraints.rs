//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::uniform::LeafUniform;
fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}
impl LeafUniform {
    pub(super) fn constrained(mut self) -> Self {
        self.constrain_shape();
        self.constrain_margins();
        self.constrain_topology();
        self.constrain_raster();
        self
    }
    fn constrain_shape(&mut self) {
        self.profile[0] = self.profile[0].clamp(0.30, 0.78);
        self.profile[1] = self.profile[1].clamp(0.06, 0.62);
        self.profile[2] = self.profile[2].clamp(0.10, 0.90);
        self.profile[3] = self.profile[3].clamp(0.20, 3.5);
        self.shape[0] = self.shape[0].clamp(0.20, 3.5);
        self.shape[1] = self.shape[1].clamp(0.0, 0.75);
        self.shape[2] = self.shape[2].clamp(0.0, 0.75);
        self.shape[3] = self.shape[3].clamp(-0.45, 0.45);

        let horizontal_clearance = (0.92 - self.profile[1]).max(0.08);
        self.axis[0] = self.axis[0].clamp(-horizontal_clearance, horizontal_clearance);
        self.axis[1] = self.axis[1].clamp(0.02, (0.94 - self.profile[0]).max(0.02));
        self.axis[2] = self.axis[2].clamp(0.002, 0.035);
        self.axis[3] = self.axis[3].clamp(0.002, 0.035);

        self.notches[0] = self.notches[0].clamp(0.0, 0.30);
        self.notches[2] = self.notches[2].clamp(0.0, 0.30);
        let notch_budget = self.profile[1] * 0.38;
        self.notches[1] = self.notches[1].clamp(0.0, notch_budget);
        self.notches[3] = self.notches[3].clamp(0.0, notch_budget);
        let combined_notches = self.notches[0] + self.notches[2];
        if combined_notches > 0.48 {
            let scale = 0.48 / combined_notches;
            self.notches[0] *= scale;
            self.notches[2] *= scale;
        }

        self.lobes[0] = self.lobes[0].clamp(0.0, 16.0);
        self.lobes[1] = self.lobes[1].clamp(0.0, 0.76);
        self.lobes[2] = self.lobes[2].clamp(0.15, 4.0);
        self.lobes[3] = self.lobes[3].clamp(-0.35, 0.35);
        self.teeth[0] = self.teeth[0].clamp(0.0, 40.0);
        self.teeth[1] = self.teeth[1].clamp(0.0, 0.24);
        self.teeth[2] = self.teeth[2].clamp(0.20, 5.0);
        self.teeth[3] = self.teeth[3].clamp(0.10, 0.90);

        self.veins[0] = self.veins[0].clamp(0.0, 14.0);
        self.veins[1] = self.veins[1].clamp(0.06, 0.42);
        self.veins[2] = self.veins[2].clamp(0.58, 0.94);
        self.veins[3] = self.veins[3].clamp(0.20, 1.0);
        self.vein_style[0] = self.vein_style[0].clamp(-0.20, 0.20);
        self.vein_style[1] = self.vein_style[1].clamp(-0.25, 0.25);
        self.vein_style[2] = self.vein_style[2].clamp(0.002, 0.022);
        self.vein_style[3] = self.vein_style[3].clamp(0.05, 0.65);
        // Secondary ribs may not be thicker than their owning midrib or narrow organ.
        self.vein_style[2] = self.vein_style[2]
            .min(self.axis[3] * 0.85)
            .min(self.organs[3].max(0.018) * 0.18)
            .max(0.002);
    }
    fn constrain_margins(&mut self) {
        for scale in &mut self.lobe_grade[..3] {
            *scale = scale.clamp(0.20, 1.30);
        }
        self.lobe_grade[3] = self.lobe_grade[3].clamp(-0.20, 0.20);
        self.margin_style[0] = self.margin_style[0].clamp(1.0, 4.0);
        self.margin_style[1] = clamp01(self.margin_style[1]);
        self.margin_style[2] = self.margin_style[2].clamp(-1.0, 1.0);
        self.margin_style[3] = self.margin_style[3].clamp(0.0, 0.12);
        for value in &mut self.venation {
            *value = clamp01(*value);
        }

        for value in &mut self.topology {
            *value = clamp01(*value);
        }
        // A fan is a radial blade organization. Treating fan weight without a
        // corresponding radial base can otherwise create detached radial organs.
        if self.topology[2] > 0.01 {
            self.topology[0] = self.topology[0].max(self.topology[2]);
            self.topology[1] = 0.0;
            self.lobes[1] = 0.0;
            self.teeth[1] = 0.0;
            self.landmarks[1] = 0.0;
            self.landmarks[2] = 0.0;
        }
        if self.topology[0] > 0.5 && self.topology[1] > 0.5 {
            self.lobes[1] = 0.0;
            self.teeth[1] = 0.0;
            self.landmarks = [0.0; 4];
            self.notches = [0.0; 4];
        }
        self.organs[0] = self.organs[0].clamp(1.0, 11.0);
        self.organs[1] = self.organs[1].clamp(0.20, 1.50);
        self.organs[2] = self.organs[2].clamp(0.08, 1.15);
        self.organs[3] = self.organs[3].clamp(0.018, 0.32);
        self.organ_style[0] = self.organ_style[0].clamp(0.0, 0.30);
        self.organ_style[1] = self.organ_style[1].clamp(0.35, 1.30);
        self.organ_style[2] = clamp01(self.organ_style[2]);
        self.organ_style[3] = clamp01(self.organ_style[3]);
        self.hierarchy[0] = clamp01(self.hierarchy[0]);
        self.hierarchy[1] = self.hierarchy[1].clamp(0.0, 6.0);
        self.hierarchy[2] = self.hierarchy[2].clamp(0.0, 10.0);
        self.hierarchy[3] = self.hierarchy[3].clamp(0.10, 0.52);
        self.landmarks[0] = self.landmarks[0].clamp(-0.15, 0.15);
        self.landmarks[1] = self.landmarks[1].clamp(0.0, 0.65);
        self.landmarks[2] = self.landmarks[2].clamp(-1.0, 1.0);
        self.landmarks[3] = clamp01(self.landmarks[3]);
        for value in &mut self.planar {
            *value = clamp01(*value);
        }
        self.planar[2] = self.planar[2].clamp(0.0, 0.42);
    }
    fn constrain_topology(&mut self) {
        // Architecture precedence prevents incompatible skeleton systems from being
        // overprinted when callers freely combine topology controls.
        if self.hierarchy[0] > 0.02 {
            self.hierarchy[1] = self.hierarchy[1].clamp(2.0, 6.0);
            self.hierarchy[2] = self.hierarchy[2].clamp(2.0, 10.0);
            self.topology = [0.0; 4];
            self.planar = [0.0; 4];
            self.landmarks = [0.0; 4];
            self.notches = [0.0; 4];
            self.organs[2] = self.organs[2].clamp(0.16, 0.24);
            self.organs[3] = self.organs[3].clamp(0.045, 0.09);
            self.hierarchy[3] = self.hierarchy[3].max(0.24);
        } else {
            self.hierarchy = [0.0; 4];
            if self.planar[2] > 0.01 {
                self.topology = [1.0, 0.0, 0.0, 0.0];
                self.planar[0] = 0.0;
                self.planar[1] = 0.0;
                self.organ_style[2] = 1.0;
            } else if self.planar[1] > 0.01 {
                self.topology[0] = 1.0;
                self.topology[2] = 0.0;
                self.planar[0] = 0.0;
                self.organs[0] = self.organs[0].clamp(5.0, 11.0);
                self.organs[3] = self.organs[3].min(0.12);
            } else if self.planar[0] > 0.01 {
                self.topology = [0.0, 1.0, 0.0, 0.0];
                self.organs[0] = self.organs[0].clamp(4.0, 11.0);
                self.organs[1] = self.organs[1].max(0.55);
                self.organs[2] = self.organs[2].clamp(0.28, 0.55);
                self.organs[3] = self.organs[3].clamp(0.070, 0.10);
                self.lobes[1] = 0.0;
                self.teeth[1] = 0.0;
                self.landmarks = [0.0; 4];
                self.notches = [0.0; 4];
            } else if self.topology[0] >= 0.5 && self.topology[1] > 0.01 {
                // Radial compound organs need enough projected area for their
                // petiolule/blade contact to survive rasterization. The shader's
                // fractional presence envelope still introduces them continuously.
                self.organs[2] = self.organs[2].max(0.16);
                self.organs[3] = self.organs[3].max(0.040);
                self.organ_style[2] = 0.0;
                self.organ_style[3] = 0.0;
            } else if self.topology[1] > 0.01 && self.topology[0] < 0.5 {
                self.organs[2] = self.organs[2].max(0.22);
                self.organs[3] = self.organs[3].max(0.060);
                self.organ_style[1] = self.organ_style[1].max(0.60);
            }
        }
    }
    fn constrain_raster(&mut self) {
        // Keep organs inside the card and prevent adjacent dense organs from becoming
        // unresolvable slivers. Touching blades are allowed; inverted geometry is not.
        if self.topology[0] < 0.5 {
            self.organs[2] = self.organs[2].min(0.88 - self.organ_style[0]);
        }
        self.organs[3] = self.organs[3].min(self.organs[2] * 0.72);
        self.hierarchy[3] = self.hierarchy[3].min(0.62);

        self.render[0] = self.render[0].max(1.0);
        self.render[1] = self.render[1].max(1.0);
        let detail_resolution = self.render[0].min(self.render[1]);
        if self.topology[0] >= 0.5 && self.topology[1] > 0.01 {
            // A separated radial leaflet is owned by its radial primary. Ensure
            // that primary survives sampling so the compound leaf cannot degrade
            // into floating opaque islands at small output sizes.
            self.vein_style[2] =
                self.vein_style[2].max(1.25 / (detail_resolution * self.topology[0].max(0.01)));
        }
        self.lobes[0] = self.lobes[0].min((detail_resolution / 9.0).max(2.0));
        self.teeth[0] = self.teeth[0].min((detail_resolution / 5.0).max(3.0));
        if self.lobes[0] > 0.0 {
            let pixels_per_lobe = detail_resolution / self.lobes[0];
            self.lobes[1] *= (pixels_per_lobe / 10.0).clamp(0.0, 1.0);
        }
        if self.teeth[0] > 0.0 {
            let pixels_per_tooth = detail_resolution / self.teeth[0];
            self.teeth[1] *= (pixels_per_tooth / 7.0).clamp(0.0, 1.0);
        }
    }
}
