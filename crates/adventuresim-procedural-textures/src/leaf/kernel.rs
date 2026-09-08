//! Shared leaf morphology adapted from adventure-simulator-group/leaves.
use super::{math::*, uniform::LeafUniform};
use bevy::math::Vec4;
pub(super) struct Kernel {
    pub(super) profile: Vec4,
    pub(super) shape: Vec4,
    pub(super) axis: Vec4,
    pub(super) notches: Vec4,
    pub(super) lobes: Vec4,
    pub(super) teeth: Vec4,
    pub(super) veins: Vec4,
    pub(super) vein_style: Vec4,
    pub(super) lobe_grade: Vec4,
    pub(super) margin_style: Vec4,
    pub(super) venation: Vec4,
    pub(super) topology: Vec4,
    pub(super) organs: Vec4,
    pub(super) organ_style: Vec4,
    pub(super) hierarchy: Vec4,
    pub(super) landmarks: Vec4,
    pub(super) planar: Vec4,
    pub(super) render: Vec4,
}
impl From<LeafUniform> for Kernel {
    fn from(u: LeafUniform) -> Self {
        Self {
            profile: Vec4::from_array(u.profile),
            shape: Vec4::from_array(u.shape),
            axis: Vec4::from_array(u.axis),
            notches: Vec4::from_array(u.notches),
            lobes: Vec4::from_array(u.lobes),
            teeth: Vec4::from_array(u.teeth),
            veins: Vec4::from_array(u.veins),
            vein_style: Vec4::from_array(u.vein_style),
            lobe_grade: Vec4::from_array(u.lobe_grade),
            margin_style: Vec4::from_array(u.margin_style),
            venation: Vec4::from_array(u.venation),
            topology: Vec4::from_array(u.topology),
            organs: Vec4::from_array(u.organs),
            organ_style: Vec4::from_array(u.organ_style),
            hierarchy: Vec4::from_array(u.hierarchy),
            landmarks: Vec4::from_array(u.landmarks),
            planar: Vec4::from_array(u.planar),
            render: Vec4::from_array(u.render),
        }
    }
}
impl Kernel {
    pub(super) fn class(&self, p: Vec2) -> u8 {
        let field = self.blade_field(p);
        let inside = field >= 0.0;
        let t = clamp((self.profile.x - p.y) / (2.0 * self.profile.x), 0.0, 1.0);
        let px = 1.05 / self.render.y;
        let pinnate_compound = self.topology.y > 0.01 && self.topology.x < 0.5;
        let ordinary_simple = self.topology.x < 0.01 && self.topology.y < 0.01;
        let basal_attachment = self.notches.x > 0.0 && t < self.notches.x;
        let terminal_origin = self.axis_point(0.80);
        let rachis_end = terminal_origin
            + Vec2::new(
                0.0,
                -self.organs.z * self.organ_style.y * 0.86 * self.topology.y,
            );
        let simple_midrib = ordinary_simple
            && (inside || basal_attachment)
            && self.axis_hit(p, 0.0, 0.98, max(px, self.axis.w));
        let compound_rachis = pinnate_compound
            && (self.axis_hit(p, 0.0, 0.80, max(px, self.axis.w))
                || self.line_hit(p, terminal_origin, rachis_end, max(px, self.axis.w)));
        let midrib = simple_midrib || compound_rachis;
        let insertion = self.axis_point(clamp(self.planar.z, 0.0, 0.55));
        let petiole = self.line_hit(
            p,
            insertion,
            insertion + Vec2::new(0.0, self.axis.y + 2.0 * self.profile.x * self.planar.z),
            max(px, self.axis.z),
        );
        let petiole_bridge = self.axis_hit(
            p,
            clamp(self.planar.z, 0.0, 0.55),
            clamp(self.planar.z + 0.08, 0.0, 0.63),
            max(px * 1.5, self.axis.z),
        );
        let attachment_hub = length(p - self.axis_point(0.0)) < max(px * 4.0, self.axis.z * 2.2);
        let insertion_hub =
            self.planar.z > 0.0 && length(p - insertion) < max(px * 1.8, self.axis.z * 1.8);
        let radial =
            self.topology.x > 0.01 && self.radial_vein_hit(p) && (inside || self.topology.y > 0.01);
        let radial_attachment = self.topology.x > 0.01
            && self.axis_hit(
                p,
                0.0,
                clamp(self.organ_style.x + self.planar.z, 0.0, 0.55),
                max(px, self.axis.w),
            );
        let landmark_suppression = clamp(self.landmarks.y * 3.0, 0.0, 1.0);
        let secondary = inside
            && self.topology.y < 0.01
            && self.hierarchy.y <= 0.0
            && self.topology.x < 0.98
            && landmark_suppression < 0.5
            && self.secondary_hit(p);
        let parallel = inside && self.parallel_hit(p);
        let pinnate_local = pinnate_compound && self.pinnate_organ_vein_hit(p);
        let compound_secondary = field >= px * 1.5 && self.compound_local_secondary_hit(p);
        let basal_primary = inside && self.basal_primary_hit(p);
        let bipinnate = self.bipinnate_vein_hit(p);
        if petiole
            || petiole_bridge
            || attachment_hub
            || insertion_hub
            || radial_attachment
            || midrib
            || radial
            || bipinnate
            || pinnate_local
            || (inside && (basal_primary || secondary || parallel || compound_secondary))
        {
            return 2;
        }
        if inside {
            return 1;
        }
        0
    }
}
