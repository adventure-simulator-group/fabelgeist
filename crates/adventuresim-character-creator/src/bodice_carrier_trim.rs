//! Design openings cut from the shared anatomical fitting carrier.
//! Planes use canonical unposed Y-up metres and anatomical lateral/depth axes.
use crate::armor_frames::{FitRegion, Wearer};
use crate::cloth_torso_frame::ClothTorsoFrame;
use crate::surface_cut::{ConvexRegion, Plane, dot};
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_armor::{BodiceDesign, FittedSleeves, NecklineShape};
use fabelgeist_bvh::{Ray, TriangleBvh};
use fabelgeist_math::Vec3 as BvhPoint;

impl ClothTorsoFrame {
    pub(crate) fn carrier_openings(
        &self,
        design: &BodiceDesign,
        body: &Wearer<'_>,
        sleeves: Option<&FittedSleeves>,
    ) -> Result<Vec<ConvexRegion>> {
        let radius = self.breadth() * design.neckline.width.unit() * 0.5;
        let mut regions = Vec::new();
        for (side, drop) in [
            (1., design.neckline.front_drop),
            (-1., design.neckline.back_drop),
        ] {
            let mut neck = vec![self.depth_half(side)];
            for sign in [-1., 1.] {
                neck.push(Plane {
                    normal: self.lateral.map(|v| sign * v),
                    offset: sign * dot(self.neck, self.lateral) + radius,
                });
            }
            if design.neckline.shape == NecklineShape::Square {
                neck.push(Plane {
                    normal: [0., -1., 0.],
                    offset: -self.neck[1] + drop.metres(),
                });
            } else {
                for segment in 1..super::CUT_ARC_SEGMENTS {
                    let angle =
                        std::f32::consts::PI * segment as f32 / super::CUT_ARC_SEGMENTS as f32;
                    let [across, down] = super::neckline_tangent(design.neckline.shape, angle);
                    let mut normal = self.lateral.map(|v| v * across / radius);
                    normal[1] -= down / drop.metres();
                    let offset = dot(normal, self.neck) + 1.;
                    let length = dot(normal, normal).sqrt();
                    neck.push(Plane {
                        normal: normal.map(|v| v / length),
                        offset: offset / length,
                    });
                }
            }
            regions.push(neck);
        }
        if sleeves.is_some() {
            return Ok(regions);
        }
        let support = body
            .support_indices(FitRegion::Torso)?
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let faces = body.faces.to_vec();
        let tree = TriangleBvh::new(
            body.positions
                .iter()
                .map(|p| BvhPoint::new(p[0], p[1], p[2]))
                .collect(),
            faces,
        );
        let lateral = Vec3::from_array(self.lateral);
        let anterior = Vec3::from_array(self.anterior);
        for (ordinal, shoulder) in self.shoulders.iter().enumerate() {
            let sign = if ordinal == 0 { -1. } else { 1. };
            let bottom = shoulder[1] - design.armhole_depth.metres();
            let mut origin = Vec3::from_array(self.waist);
            origin.y = bottom;
            let cast = |origin: Vec3, direction: Vec3| -> Option<Vec3> {
                let ray = Ray::new(
                    BvhPoint::new(origin.x, origin.y, origin.z),
                    BvhPoint::new(direction.x, direction.y, direction.z),
                );
                tree.raycast(&ray, f32::MAX)
                    .map(|(_, distance)| origin + direction * distance)
            };
            let front =
                cast(origin, anterior).context("carrier armscye has no front torso support")?;
            let back =
                cast(origin, -anterior).context("carrier armscye has no back torso support")?;
            origin += anterior * ((front + back) * 0.5 - origin).dot(anterior);
            let bank = cast(origin,lateral*sign).with_context(|| format!("carrier armscye has no lateral torso support: side={sign} origin={origin:?} front={front:?} back={back:?} bottom={bottom}"))?;
            let lower = sign * bank.dot(lateral);
            let medial = sign * dot(*shoulder, self.lateral) - design.armhole_width.metres();
            let top = support
                .iter()
                .map(|&v| body.positions[v][1])
                .fold(shoulder[1], f32::max);
            ensure!(
                top > bottom && lower > medial,
                "carrier armscye has no positive shoulder span"
            );
            let slope = (medial - lower) / (top - bottom);
            let mut normal = self.lateral.map(|v| -sign * v);
            normal[1] += slope;
            let length = dot(normal, normal).sqrt();
            regions.push(vec![Plane {
                normal: normal.map(|v| v / length),
                offset: (-lower + slope * bottom) / length,
            }]);
        }
        Ok(regions)
    }
}
