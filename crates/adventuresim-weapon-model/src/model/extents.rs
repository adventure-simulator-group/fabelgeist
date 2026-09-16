//! Local axial extents of authored component shapes.
use super::*;
fn profile_range(profile: &[[Metres; 2]]) -> Result<[f64; 2], String> {
    if profile.len() < 2 {
        return Err("radial profile requires at least two stations".into());
    }
    Ok([
        profile
            .iter()
            .map(|p| p[0].get())
            .fold(f64::INFINITY, f64::min),
        profile
            .iter()
            .map(|p| p[0].get())
            .fold(f64::NEG_INFINITY, f64::max),
    ])
}

impl Shape {
    /// Assembly attachments use their declared node; other shapes use axial seats.
    pub(super) fn attachment_anchor(
        &self,
        at: AttachmentAnchor,
        range: [f64; 2],
    ) -> Result<Point, String> {
        if at == AttachmentAnchor::HeelCenter {
            return match self {
                Self::Blade(p) => Ok(p.heel_center()),
                _ => Err("heel-center requires a generic blade".into()),
            };
        }
        if let Self::GuardAssembly(p) = self {
            return p
                .nodes
                .get(
                    p.anchor_node
                        .as_ref()
                        .ok_or("guard assembly needs anchorNode")?,
                )
                .ok_or_else(|| "missing guard anchor node".to_owned())
                .map(|point| point.map(Metres::get));
        }
        let y = match at {
            AttachmentAnchor::Base => range[0],
            AttachmentAnchor::Center => (range[0] + range[1]) / 2.0,
            AttachmentAnchor::Top => range[1],
            AttachmentAnchor::Origin => 0.0,
            AttachmentAnchor::HeelCenter => unreachable!(),
        };
        Ok([0.0, y, 0.0])
    }

    pub(super) fn range(&self) -> Result<[f64; 2], String> {
        Ok(match self {
            Self::BentBar(p) => match p.centerline {
                BarCenterline::Opposed { sweep, .. } => [-sweep.get().abs(), sweep.get().abs()],
                BarCenterline::Arch { length, .. } => [0.0, length.get()],
            },
            Self::SpatialTube(p) => {
                if p.points.is_empty() {
                    return Err("spatial member needs points".into());
                }
                [
                    p.points
                        .iter()
                        .map(|p| p[1].get())
                        .fold(f64::INFINITY, f64::min),
                    p.points
                        .iter()
                        .map(|p| p[1].get())
                        .fold(f64::NEG_INFINITY, f64::max),
                ]
            }
            Self::LoftedBlade(p) => [0.0, p.length.get()],
            Self::Shaft(p) => [0.0, p.length.get()],
            Self::Crossbow(p) => [0.0, p.length.get()],
            Self::CrossbowBolt(p) => [0.0, p.length.get()],
            Self::BoltQuiver(p) => [0.0, p.length.get()],
            Self::Firearm(p) => [0.0, p.length.get()],
            Self::BallPouch(p) => [0.0, p.height.get()],
            Self::LeadBall(p) => [-p.radius.get(), p.radius.get()],
            Self::ArcheryBow(p) => [
                -p.length.get() * (1.0 - p.upper_ratio.get()),
                p.length.get() * p.upper_ratio.get(),
            ],
            Self::Arrow(p) => [0.0, p.length.get()],
            Self::ArrowQuiver(p) => [0.0, p.length.get()],
            Self::RoundShield(p) => [-p.radius.get(), p.radius.get()],
            Self::ShapedShield(p) => [
                -p.height.get() / 2.0 - p.bottom_depth.get(),
                p.height.get() / 2.0 + p.top_depth.get(),
            ],
            Self::Grip(p) => [0.0, p.length.get()],
            Self::OvalGrip(p) => [0.0, p.length.get()],
            Self::ProfileGrip(p) => [0.0, p.length.get()],
            Self::SlabGrip(p) => [0.0, p.length.get()],
            Self::Blade(p) => [0.0, p.length.get()],
            Self::SectionBlade(p) => [0.0, p.length.get()],
            Self::DiamondBlade(p) => [0.0, p.length.get()],
            Self::Spear(p) => [
                -p.socket.as_ref().map_or(0.0, |s| s.length.get()),
                p.length.get(),
            ],
            Self::Fork(p) => [0.0, p.length.get()],
            Self::Partisan(p) => [0.0, p.length.get()],
            Self::Glaive(p) => [0.0, p.length.get()],
            Self::Bill(p) => [0.0, p.length.get()],
            Self::Sleeve(p) => [0.0, p.length.get()],
            Self::Guard(p) => [-p.height.get() / 2.0, p.height.get() / 2.0],
            Self::RingGuard(p) => [-p.radius.get(), p.radius.get()],
            Self::FigureEight(p) => {
                let h = p.height.map_or(p.width.get() * 0.15, Metres::get);
                [-h, h]
            }
            Self::KnuckleBow(p) => [0.0, p.length.get()],
            Self::Tube(p) => [0.0, p.points.last().ok_or("tube needs points")?[1].get()],
            Self::Pommel(p) => pommel_range(p)?,
            Self::GuardAssembly(p) => {
                if p.nodes.is_empty() {
                    return Err("guard assembly needs nodes".into());
                }
                [
                    p.nodes
                        .values()
                        .map(|p| p[1].get())
                        .fold(f64::INFINITY, f64::min),
                    p.nodes
                        .values()
                        .map(|p| p[1].get())
                        .fold(f64::NEG_INFINITY, f64::max),
                ]
            }
            Self::Socket(p) => profile_range(&p.profile)?,
            Self::Mace(p) => [
                -p.length.get() / 2.0,
                p.length.get() / 2.0 + p.crown_length.map_or(0.0, Metres::get),
            ],
            Self::Collar(p) => [-p.width.get() / 2.0, p.width.get() / 2.0],
            Self::Box(p) => [-p.size[1].get() / 2.0, p.size[1].get() / 2.0],
            Self::Axe(_)
            | Self::FacetedBeak(_)
            | Self::Pick(_)
            | Self::Beak(_)
            | Self::Hammer(_) => [0.0, 0.0],
        })
    }
}

fn pommel_range(p: &PommelParameters) -> Result<[f64; 2], String> {
    Ok({
        if p.construction == PommelConstruction::Lathed
            || (p.construction == PommelConstruction::Composite
                && p.base_construction == Some(PommelBaseConstruction::Lathed))
        {
            profile_range(p.profile.as_deref().ok_or("lathed pommel needs profile")?)?
        } else {
            [
                0.0,
                p.height.map_or(0.06, Metres::get) * p.length_scale.map_or(1.0, Ratio::get),
            ]
        }
    })
}
