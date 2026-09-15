//! Dispatch from typed manufacturing shapes to the shared solid constructors.
use super::*;

pub(super) fn construct(
    resolved: &ResolvedComponent,
    detail: Detail,
) -> Result<Vec<PartSource>, String> {
    ComponentConstructor { resolved, detail }.construct()
}

struct ComponentConstructor<'a> {
    resolved: &'a ResolvedComponent,
    detail: Detail,
}
impl ComponentConstructor<'_> {
    fn one(&self, solid: Solid) -> Result<Vec<PartSource>, String> {
        Ok(vec![PartSource::new(
            solid,
            self.resolved.component.material.unwrap_or(Material::Steel),
            &self.resolved.label,
            &self.resolved.id,
        )])
    }
    fn construct(&self) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let detail = self.detail;
        let c = &resolved.component;
        match &c.shape {
            Shape::BentBar(p) => self.one(bent_bar::bar(p, detail)?),
            Shape::SpatialTube(p) => self.spatial_tube(p),
            Shape::LoftedBlade(p) => {
                let mut parts = self.one(lofted_blade::blade(p, detail)?)?;
                for part in &mut parts {
                    part.smoothing_cosine = 0.98;
                }
                Ok(parts)
            }
            Shape::Shaft(p) => self.shaft(p),
            Shape::Spear(p) => {
                let mut parts = self.one(spears::spear(p, detail)?)?;
                if p.socket.is_some() {
                    for part in &mut parts {
                        part.surface_smoothing.insert(
                            1,
                            output::SurfaceSmoothing {
                                group: 1,
                                cosine: 0.25,
                            },
                        );
                        part.surface_smoothing.insert(
                            2,
                            output::SurfaceSmoothing {
                                group: 1,
                                cosine: spear_socket::BLADE_CREASE_COSINE,
                            },
                        );
                    }
                }
                Ok(parts)
            }
            Shape::ArcheryBow(p) => archery::bow(resolved, p, detail),
            Shape::Arrow(p) => archery::arrow(resolved, p, detail),
            Shape::ArrowQuiver(p) => archery::quiver(resolved, p, detail),
            Shape::Crossbow(p) => crossbows::crossbow(resolved, p, detail),
            Shape::Firearm(p) => firearms::firearm(resolved, p, detail),
            Shape::LeadBall(p) => Ok(ammunition::ball(resolved, p, detail)),
            Shape::BallPouch(p) => ammunition::pouch(resolved, p, detail),
            Shape::CrossbowBolt(p) => bolts::bolt(resolved, p, detail),
            Shape::BoltQuiver(p) => bolts::quiver(resolved, p, detail),
            Shape::Mace(p) => maces::mace(resolved, p, detail),
            Shape::Guard(p) => guards::guard(resolved, p, detail),
            Shape::GuardAssembly(p) => guards::assembly(resolved, p, detail),
            Shape::KnuckleBow(p) => guards::knuckle(resolved, p, detail),
            Shape::RingGuard(p) => self.one(guards::ring(p, detail)?),
            Shape::FigureEight(p) => self.one(guards::figure_eight(p, detail)?),
            Shape::Tube(p) => self.tube(p),
            Shape::Blade(p) => self.one(blades::blade(p, detail)?),
            Shape::SectionBlade(p) => self.one(blades::section_blade(p, detail)?),
            Shape::DiamondBlade(p) => self.one(blades::diamond_blade(p, detail)),
            Shape::Axe(p) => self.one(blades::axe(p, detail)?),
            Shape::Fork(p) => self.one(blades::fork(p, detail)?),
            Shape::Partisan(p) => self.one(blades::partisan(p, detail)?),
            Shape::Glaive(p) => self.one(blades::glaive(p, detail)?),
            Shape::Bill(p) => self.one(polls::bill(p, detail)?),
            Shape::Hammer(p) => self.one(polls::hammer(p, detail)?),
            Shape::Beak(p) => self.one(polls::beak(p, detail)?),
            Shape::FacetedBeak(p) => self.one(polls::faceted_beak(p, detail)?),
            Shape::Pick(p) => self.pick(p),
            Shape::Socket(p) => self.socket(p),
            Shape::Pommel(p) => pommels::pommel(resolved, p, detail),
            Shape::Box(p) => self.one(Solid::cuboid(p.size.map(Metres::get), detail)?),
            Shape::Grip(p) => self.grip(p),
            Shape::OvalGrip(p) => self.oval_grip(p),
            Shape::SlabGrip(p) => self.slab_grip(p),
            Shape::Collar(p) => self.collar(p),
            Shape::Sleeve(p) => self.sleeve(p),
            Shape::RoundShield(_) | Shape::ShapedShield(_) => shields::Shield::from_shape(&c.shape)
                .unwrap()
                .construct(resolved, detail),
        }
    }
    fn socket(&self, p: &SocketParameters) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let detail = self.detail;
        let profile: Vec<_> = p
            .profile
            .iter()
            .map(|point| point.map(Metres::get))
            .collect();
        if let Some(radius) = resolved.shaft_contact {
            self.one(Solid::hollow_socket(
                &profile,
                &vec![radius; profile.len()],
                None,
                detail,
            ))
        } else if let Some(wall) = p.wall {
            let inner: Vec<_> = profile.iter().map(|p| p[1] - wall.get()).collect();
            if inner.iter().any(|&r| r <= 0.0) {
                return Err("socket wall leaves no bore".into());
            }
            self.one(Solid::hollow_socket(
                &profile,
                &inner,
                p.segments.map(|n| n.0 as usize),
                detail,
            ))
        } else {
            self.one(Solid::lathe(
                &profile,
                p.segments.map_or(14, |n| n.0 as usize),
                1.0,
                false,
                detail,
            )?)
        }
    }
    fn grip(&self, p: &GripParameters) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let c = &resolved.component;
        let detail = self.detail;
        let mut parts = vec![PartSource::new(
            Solid::lathe(
                &[
                    [0.0, p.radius.get() * p.bottom_scale.map_or(1.0, Ratio::get)],
                    [
                        p.length.get(),
                        p.radius.get() * p.top_scale.map_or(1.0, Ratio::get),
                    ],
                ],
                p.segments.map_or(14, |n| n.0 as usize),
                1.0,
                false,
                detail,
            )?,
            c.material.unwrap_or(Material::Leather),
            &resolved.label,
            &resolved.id,
        )];
        let turns = p.wraps.map_or(0, |n| n.0);
        for turn in 1..turns {
            parts.push(PartSource::new(
                Solid::lathe(
                    &[
                        [-0.002, p.radius.get() * 1.018],
                        [0.002, p.radius.get() * 1.018],
                    ],
                    16,
                    1.0,
                    false,
                    detail,
                )?
                .transform(
                    [0.0; 3],
                    [0.0, p.length.get() * turn as f64 / turns as f64, 0.0],
                ),
                Material::DarkSteel,
                "grip binding",
                &resolved.id,
            ));
        }
        Ok(parts)
    }
    fn oval_grip(&self, p: &OvalGripParameters) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let detail = self.detail;
        let base = p.width.get() * p.bottom_scale.map_or(1.0, Ratio::get) / 2.0;
        self.one(Solid::lathe(
            &[
                [0.0, resolved.grip_seat_radius.unwrap_or(base)],
                [(p.length.get() * 0.15).min(0.016), base],
                [
                    p.length.get(),
                    p.width.get() * p.top_scale.map_or(1.0, Ratio::get) / 2.0,
                ],
            ],
            p.segments.map_or(16, |n| n.0 as usize),
            p.thickness.get() / p.width.get(),
            true,
            detail,
        )?)
    }
    fn slab_grip(&self, p: &SlabGripParameters) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let c = &resolved.component;
        let detail = self.detail;
        if p.construction == Some(SlabGripConstruction::Homogeneous) {
            return self.one(
                Solid::cuboid(
                    [
                        p.width.get(),
                        p.length.get(),
                        p.thickness.get() + 2.0 * p.scale_thickness.map_or(0.006, Metres::get),
                    ],
                    detail,
                )?
                .transform([0.0; 3], [0.0, p.length.get() / 2.0, 0.0]),
            );
        }
        let mut parts = vec![PartSource::new(
            Solid::cuboid([p.width.get(), p.length.get(), p.thickness.get()], detail)?
                .transform([0.0; 3], [0.0, p.length.get() / 2.0, 0.0]),
            Material::DarkSteel,
            &format!("{} tang", resolved.label),
            &resolved.id,
        )];
        let scale = p.scale_thickness.map_or(0.006, Metres::get);
        for side in [-1.0, 1.0] {
            parts.push(PartSource::new(
                Solid::cuboid([p.width.get() * 0.92, p.length.get() * 0.92, scale], detail)?
                    .transform(
                        [0.0; 3],
                        [
                            0.0,
                            p.length.get() / 2.0,
                            side * (p.thickness.get() + scale) / 2.0,
                        ],
                    ),
                c.material.unwrap_or(Material::Wood),
                &format!("{} scale", resolved.label),
                &resolved.id,
            ));
        }
        Ok(parts)
    }
    fn collar(&self, p: &CollarParameters) -> Result<Vec<PartSource>, String> {
        let detail = self.detail;
        let w = p.width.get();
        let r = p.radius.get();
        self.one(Solid::lathe(
            &[
                [-w / 2.0, r * 0.88],
                [-w * 0.32, r],
                [w * 0.32, r],
                [w / 2.0, r * 0.88],
            ],
            p.segments.map_or(16, |n| n.0 as usize),
            1.0,
            false,
            detail,
        )?)
    }
    fn sleeve(&self, p: &SleeveParameters) -> Result<Vec<PartSource>, String> {
        let resolved = self.resolved;
        let detail = self.detail;
        let profile = [
            [0.0, p.radius.get()],
            [
                p.length.get(),
                p.top_radius.map_or(p.radius.get() * 0.88, Metres::get),
            ],
        ];
        if let Some(radius) = resolved.shaft_contact {
            self.one(Solid::hollow_socket(
                &profile,
                &[radius, radius],
                None,
                detail,
            ))
        } else {
            self.one(Solid::lathe(
                &profile,
                p.segments.map_or(16, |n| n.0 as usize),
                1.0,
                false,
                detail,
            )?)
        }
    }
    fn spatial_tube(&self, p: &SpatialTubeParameters) -> Result<Vec<PartSource>, String> {
        let detail = self.detail;
        self.one(Solid::sweep(
            &p.points
                .iter()
                .map(|p| p.map(Metres::get))
                .collect::<Vec<_>>(),
            &Sweep {
                width: p.radius.get() * 2.0,
                depth: p.radius.get() * 2.0,
                radial_segments: p.radial_segments.0 as usize,
                ..Sweep::default()
            },
            detail,
        )?)
    }
    fn shaft(&self, p: &Shaft) -> Result<Vec<PartSource>, String> {
        let detail = self.detail;
        let mut parts = self.one(Solid::lathe(
            &p.profile(),
            p.segments.map_or(16, |n| n.0 as usize),
            1.0,
            true,
            detail,
        )?)?;
        parts.extend(shaft_wrapping::parts(
            p,
            &self.resolved.id,
            &self.resolved.label,
            detail,
        )?);
        Ok(parts)
    }
    fn pick(&self, p: &PickParameters) -> Result<Vec<PartSource>, String> {
        let detail = self.detail;
        self.one(
            Solid::lathe(
                &[[0.0, p.radius.get()], [p.length.get(), 0.0]],
                12,
                1.0,
                true,
                detail,
            )?
            .transform(
                [0.0, 0.0, -90.0 * p.direction.map_or(-1.0, Direction::sign)],
                [0.0; 3],
            ),
        )
    }
    fn tube(&self, p: &TubeParameters) -> Result<Vec<PartSource>, String> {
        let detail = self.detail;
        self.one(guards::tube(
            &p.points
                .iter()
                .map(|v| v.map(Metres::get))
                .collect::<Vec<_>>(),
            p.radius.get(),
            p.radial_segments.map_or(8, |n| n.0 as usize),
            detail,
        )?)
    }
}
