//! Medieval buildings: round and square towers, curtain walls with
//! battlements, timber-framed houses, a castle and a village below it.
//!
//! Everything stands on a ground given as a height function. A building's
//! base goes a foundation's depth below the lowest ground under it, so that
//! on a slope no side of it floats; its heights are measured from the ground
//! at its middle.

use crate::mesh::{Material, cross, normalize, sub};
use crate::solid::Solid;
use crate::structure::{Feature, FeatureKind, ROUND_SIDES, Structure};

/// The ground: its height at `(x, z)`.
pub type Ground<'a> = &'a dyn Fn(f32, f32) -> f32;

/// How deep foundations go below the lowest ground under a building.
const FOUNDATION: f32 = 0.6;

/// Merlons: how wide, how deep, how tall, and how far apart their middles.
const MERLON: [f32; 3] = [0.8, 0.5, 1.0];
const MERLON_PITCH: f32 = 1.5;

/// SplitMix64, for the variety between buildings.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x2545_F491_4F6C_DD1D)
    }

    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}

/// Where a building of horizontal `reach` around `(x, z)` has its base: a
/// foundation below the lowest ground under it.
pub fn footing(ground: Ground, x: f32, z: f32, reach: f32) -> f32 {
    let mut low = ground(x, z);
    for k in 0..12 {
        let t = std::f32::consts::TAU * k as f32 / 12.0;
        low = low.min(ground(x + reach * t.cos(), z + reach * t.sin()));
    }
    low - FOUNDATION
}

/// A round tower.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundTower {
    pub radius: f32,
    /// Above the ground at its middle, to the top of the masonry.
    pub height: f32,
    /// A conical slate roof; without, a battlemented top.
    pub roof: bool,
    /// Its battlements broken and half fallen.
    pub ruined: bool,
}

/// A round tower standing at `(x, z)`.
pub fn round_tower(ground: Ground, x: f32, z: f32, tower: RoundTower, seed: u64) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let r = tower.radius;
    let base = footing(ground, x, z, r * 1.1);
    let floor = ground(x, z);
    let top = floor + tower.height;
    let tint = rng.range(0.88, 1.08);
    // A battered plinth, then the drum.
    s.drum(
        [x, base, z],
        r * 1.1,
        floor + 1.4 - base,
        Material::Stone,
        tint * 0.94,
    );
    s.drum([x, base, z], r, top - base, Material::Stone, tint);
    // A string course a little below the top.
    s.trim_drum(
        [x, top - 1.6, z],
        r + 0.12,
        0.35,
        Material::Stone,
        tint * 0.9,
    );
    if tower.roof {
        s.cone(
            [x, top, z],
            r + 0.4,
            r * 1.7,
            Material::Slate,
            rng.range(0.9, 1.1),
        );
    } else {
        let ring = r - MERLON[1] * 0.5;
        let count = ((std::f32::consts::TAU * ring / MERLON_PITCH).floor() as usize).max(4);
        for i in 0..count {
            let t = std::f32::consts::TAU * (i as f32 + 0.5) / count as f32;
            let height = if tower.ruined {
                if rng.next() < 0.35 {
                    continue;
                }
                MERLON[2] * rng.range(0.3, 1.0)
            } else {
                MERLON[2]
            };
            // Each merlon square to the drum: its width runs round it.
            let at = [x + ring * t.cos(), top, z - ring * t.sin()];
            s.block(
                at,
                [MERLON[1] * 0.5, MERLON[0] * 0.5],
                height,
                t,
                Material::Stone,
                tint,
            );
        }
    }
    // Arrow slits, staggered up the drum.
    let mut y = floor + 2.5;
    let mut turn = rng.range(0.0, std::f32::consts::TAU);
    while y < top - 2.5 {
        for k in 0..4 {
            let t = turn + std::f32::consts::FRAC_PI_2 * k as f32;
            let out = [t.cos(), 0.0, -t.sin()];
            let at = [x + r * out[0], y, z + r * out[2]];
            s.panel(at, out, 0.18, 1.0, 0.03, Material::Opening, 1.0);
        }
        y += 3.2;
        turn += 0.6;
    }
    s.features.push(Feature {
        kind: FeatureKind::Tower,
        base: [x, floor, z],
        half: [r, r],
        yaw: 0.0,
        height: tower.height,
        round: true,
    });
    s
}

impl Structure {
    /// A drum drawn but not solid: a moulding round a tower.
    fn trim_drum(
        &mut self,
        base: [f32; 3],
        radius: f32,
        height: f32,
        material: Material,
        tint: f32,
    ) {
        let ring = |y: f32| -> Vec<([f32; 3], [f32; 3])> {
            (0..=ROUND_SIDES)
                .map(|i| {
                    let t = std::f32::consts::TAU * i as f32 / ROUND_SIDES as f32;
                    let (s, c) = t.sin_cos();
                    (
                        [base[0] + radius * c, base[1] + y, base[2] - radius * s],
                        [c, 0.0, -s],
                    )
                })
                .collect()
        };
        self.mesh
            .smooth_band(&ring(0.0), &ring(height), material, tint);
        let top: Vec<[f32; 3]> = ring(height)
            .into_iter()
            .take(ROUND_SIDES)
            .map(|(p, _)| p)
            .collect();
        self.mesh.polygon(&top, material, tint);
    }

    /// Merlons along the top of a wall, from `from` to `to` (local to
    /// `frame`, at its top), set in from the edge they stand on.
    #[allow(clippy::too_many_arguments)]
    fn merlons(
        &mut self,
        frame: &Solid,
        from: [f32; 2],
        to: [f32; 2],
        top: f32,
        ruined: bool,
        rng: &mut Rng,
        tint: f32,
    ) {
        let along = [to[0] - from[0], to[1] - from[1]];
        let length = (along[0] * along[0] + along[1] * along[1]).sqrt();
        if length < MERLON[0] {
            return;
        }
        let count = ((length / MERLON_PITCH).floor() as usize).max(1);
        let yaw = frame.yaw + (-along[1]).atan2(along[0]);
        for i in 0..count {
            let t = (i as f32 + 0.5) / count as f32;
            let local = [from[0] + along[0] * t, top, from[1] + along[1] * t];
            let height = if ruined {
                if rng.next() < 0.35 {
                    continue;
                }
                MERLON[2] * rng.range(0.3, 1.0)
            } else {
                MERLON[2]
            };
            let at = frame.world(local);
            self.block(
                at,
                [MERLON[0] * 0.5, MERLON[1] * 0.5],
                height,
                yaw,
                Material::Stone,
                tint,
            );
        }
    }

    /// A strip of timber laid flat on a face: from `a` to `b`, `width` wide,
    /// standing `lift` proud of the face along its outward normal `out`.
    fn timber(&mut self, a: [f32; 3], b: [f32; 3], width: f32, out: [f32; 3], tint: f32) {
        let along = normalize(sub(b, a));
        let side = normalize(cross(out, along));
        let lift = 0.04;
        let o = [out[0] * lift, out[1] * lift, out[2] * lift];
        let w = width * 0.5;
        let p = |p: [f32; 3], s: f32| {
            [
                p[0] + o[0] + side[0] * s,
                p[1] + o[1] + side[1] * s,
                p[2] + o[2] + side[2] * s,
            ]
        };
        self.mesh.polygon(
            &[p(a, -w), p(b, -w), p(b, w), p(a, w)],
            Material::Timber,
            tint,
        );
    }
}

/// A square tower at `(x, z)`, `half` across either way.
#[allow(clippy::too_many_arguments)]
pub fn square_tower(
    ground: Ground,
    x: f32,
    z: f32,
    half: f32,
    height: f32,
    yaw: f32,
    roof: bool,
    seed: u64,
) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let base = footing(ground, x, z, half * 1.42);
    let floor = ground(x, z);
    let top = floor + height;
    let tint = rng.range(0.88, 1.08);
    let body = s.block(
        [x, base, z],
        [half, half],
        top - base,
        yaw,
        Material::Stone,
        tint,
    );
    s.trim(
        [x, top - 1.6, z],
        [half + 0.12, half + 0.12],
        0.35,
        yaw,
        Material::Stone,
        tint * 0.9,
    );
    if roof {
        s.pyramid(
            [x, top, z],
            half + 0.35,
            half * 1.5,
            yaw,
            Material::Slate,
            rng.range(0.9, 1.1),
        );
    } else {
        let e = half - MERLON[1] * 0.5;
        for (from, to) in [
            ([-e, -e], [e, -e]),
            ([e, -e], [e, e]),
            ([e, e], [-e, e]),
            ([-e, e], [-e, -e]),
        ] {
            s.merlons(
                &body,
                from,
                to,
                height + floor - base,
                false,
                &mut rng,
                tint,
            );
        }
    }
    windows_up_faces(
        &mut s,
        &body,
        floor - base + 3.0,
        top - base - 2.5,
        0.55,
        1.1,
        &mut rng,
    );
    s.features.push(Feature {
        kind: FeatureKind::Tower,
        base: [x, floor, z],
        half: [half, half],
        yaw,
        height,
        round: false,
    });
    s
}

/// Narrow windows up each face of a block, a storey apart.
fn windows_up_faces(
    s: &mut Structure,
    body: &Solid,
    from: f32,
    to: f32,
    width: f32,
    height: f32,
    rng: &mut Rng,
) {
    let [a, _, b] = body.size;
    let faces: [([f32; 2], [f32; 2], f32); 4] = [
        ([a, 0.0], [1.0, 0.0], b),
        ([-a, 0.0], [-1.0, 0.0], b),
        ([0.0, b], [0.0, 1.0], a),
        ([0.0, -b], [0.0, -1.0], a),
    ];
    let mut y = from;
    while y < to {
        for (middle, out, span) in faces {
            let count = ((span * 2.0 / 3.5).floor() as usize).max(1);
            for i in 0..count {
                if rng.next() < 0.25 {
                    continue;
                }
                let u = (i as f32 + 0.5) / count as f32 * 2.0 - 1.0;
                let local = [
                    middle[0] + out[1].abs() * u * span * 0.8,
                    y,
                    middle[1] + out[0].abs() * u * span * 0.8,
                ];
                let at = body.world(local);
                let o = body.world([out[0], 0.0, out[1]]);
                let out = [o[0] - body.base[0], 0.0, o[2] - body.base[2]];
                s.panel(at, out, width, height, 0.03, Material::Opening, 1.0);
            }
        }
        y += 3.6;
    }
}

/// A stretch of curtain wall from `from` to `to`, `thickness` thick and
/// `height` above the higher of its ends' ground, battlemented on the side
/// facing `outward` (a point outside the castle).
#[allow(clippy::too_many_arguments)]
pub fn curtain_wall(
    ground: Ground,
    from: [f32; 2],
    to: [f32; 2],
    thickness: f32,
    height: f32,
    outward: [f32; 2],
    seed: u64,
) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let d = [to[0] - from[0], to[1] - from[1]];
    let length = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let middle = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5];
    let yaw = (-d[1]).atan2(d[0]);
    let samples = 8;
    let (mut low, mut high) = (f32::INFINITY, f32::NEG_INFINITY);
    for i in 0..=samples {
        let t = i as f32 / samples as f32;
        let (x, z) = (from[0] + d[0] * t, from[1] + d[1] * t);
        low = low.min(footing(ground, x, z, thickness * 0.5));
        high = high.max(ground(x, z));
    }
    let top = high + height;
    let tint = rng.range(0.9, 1.06);
    let body = s.block(
        [middle[0], low, middle[1]],
        [length * 0.5, thickness * 0.5],
        top - low,
        yaw,
        Material::Stone,
        tint,
    );
    // Battlements along the outer edge, a low parapet along the inner.
    let side = {
        let local = body.local([outward[0], 0.0, outward[1]]);
        if local[2] >= 0.0 { 1.0 } else { -1.0 }
    };
    let e = thickness * 0.5 - MERLON[1] * 0.5;
    let a = length * 0.5;
    s.merlons(
        &body,
        [-a, side * e],
        [a, side * e],
        top - low,
        false,
        &mut rng,
        tint,
    );
    s.trim(
        body.world([0.0, top - low, -side * e]),
        [a, MERLON[1] * 0.5],
        0.5,
        yaw,
        Material::Stone,
        tint * 0.95,
    );
    s.features.push(Feature {
        kind: FeatureKind::Wall,
        base: [middle[0], ground(middle[0], middle[1]), middle[1]],
        half: [length * 0.5, thickness * 0.5],
        yaw,
        height,
        round: false,
    });
    s
}

/// A house: its size and its roof.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct House {
    /// Along its ridge, and across.
    pub half_length: f32,
    pub half_width: f32,
    /// Above the ground at its middle, to the eaves.
    pub eaves: f32,
    pub roof: Material,
    /// Stone walls, no timbers: a hall.
    pub stone: bool,
}

/// A house at `(x, z)`, its ridge along `yaw`, its door on its `+z` side.
pub fn house(ground: Ground, x: f32, z: f32, yaw: f32, house: House, seed: u64) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let (a, b) = (house.half_length, house.half_width);
    let base = footing(ground, x, z, (a * a + b * b).sqrt());
    let floor = ground(x, z);
    let sill = floor + 0.5;
    let eaves = floor + house.eaves;
    let tint = rng.range(0.9, 1.08);
    s.block(
        [x, base, z],
        [a + 0.08, b + 0.08],
        sill - base,
        yaw,
        Material::Stone,
        tint * 0.92,
    );
    let wall = if house.stone {
        Material::Stone
    } else {
        Material::Plaster
    };
    let body = s.block([x, sill, z], [a, b], eaves - sill, yaw, wall, tint);
    let w = |lx: f32, ly: f32, lz: f32| body.world([lx, ly - sill, lz]);

    if !house.stone {
        // The frame: posts and studs, the sill and wall plates, a rail at
        // the upper floor and braces at the corners.
        let timber_tint = rng.range(0.8, 1.15);
        let faces = [
            ([-a, b], [a, b], [0.0, 1.0]),
            ([a, -b], [-a, -b], [0.0, -1.0]),
            ([a, b], [a, -b], [1.0, 0.0]),
            ([-a, -b], [-a, b], [-1.0, 0.0]),
        ];
        let upper = sill + (eaves - sill) * 0.48;
        for (p, q, n) in faces {
            let o = body.world([n[0], 0.0, n[1]]);
            let out = [o[0] - body.base[0], 0.0, o[2] - body.base[2]];
            let at = |t: f32, y: f32| w(p[0] + (q[0] - p[0]) * t, y, p[1] + (q[1] - p[1]) * t);
            let span = ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt();
            for y in [sill + 0.1, upper, eaves - 0.1] {
                s.timber(at(0.0, y), at(1.0, y), 0.2, out, timber_tint);
            }
            let studs = ((span / 1.1).round() as usize).max(2);
            for i in 0..=studs {
                let t = i as f32 / studs as f32;
                s.timber(
                    at(t, sill),
                    at(t, eaves),
                    if i == 0 || i == studs { 0.24 } else { 0.16 },
                    out,
                    timber_tint,
                );
            }
            // Braces from each corner post up to the rail.
            let t = 1.0 / studs as f32;
            s.timber(at(0.0, sill + 0.1), at(t, upper), 0.14, out, timber_tint);
            s.timber(
                at(1.0, sill + 0.1),
                at(1.0 - t, upper),
                0.14,
                out,
                timber_tint,
            );
        }
    }

    // The roof, overhanging the walls.
    let pitch = rng.range(0.85, 1.05);
    let (ra, rb) = (a + 0.35, b + 0.5);
    s.gable(
        [x, eaves, z],
        ra,
        rb,
        rb * pitch,
        yaw,
        house.roof,
        wall,
        rng.range(0.85, 1.1),
    );
    // A chimney through the roof at one end.
    if rng.next() < 0.7 {
        let end = if rng.next() < 0.5 { -1.0 } else { 1.0 };
        let at = body.world([end * (a - 0.5), 0.0, b * 0.3]);
        s.block(
            [at[0], eaves - 0.5, at[2]],
            [0.35, 0.35],
            rb * pitch + 1.2,
            yaw,
            Material::Stone,
            tint * 0.85,
        );
    }
    // A door and windows on the front, windows on the back.
    let front = {
        let o = body.world([0.0, 0.0, 1.0]);
        [o[0] - body.base[0], 0.0, o[2] - body.base[2]]
    };
    let back = [-front[0], 0.0, -front[2]];
    let door_at = rng.range(-0.4, 0.4) * a;
    s.panel(
        w(door_at, sill + 1.0, b),
        front,
        1.0,
        2.0,
        0.05,
        Material::Wood,
        rng.range(0.8, 1.2),
    );
    let upper_y = sill + (eaves - sill) * 0.72;
    let count = ((a * 2.0 / 2.2).floor() as usize).max(1);
    for i in 0..count {
        let u = ((i as f32 + 0.5) / count as f32 * 2.0 - 1.0) * a * 0.75;
        s.panel(
            w(u, upper_y, b),
            front,
            0.6,
            0.75,
            0.05,
            Material::Opening,
            1.0,
        );
        s.panel(
            w(u, upper_y, -b),
            back,
            0.6,
            0.75,
            0.05,
            Material::Opening,
            1.0,
        );
        if (u - door_at).abs() > 1.0 {
            s.panel(
                w(u, sill + 1.3, b),
                front,
                0.6,
                0.7,
                0.05,
                Material::Opening,
                1.0,
            );
        }
    }
    s.features.push(Feature {
        kind: if house.stone {
            FeatureKind::Hall
        } else {
            FeatureKind::House
        },
        base: [x, floor, z],
        half: [a, b],
        yaw,
        height: house.eaves,
        round: false,
    });
    s
}

/// A castle: four corner towers, curtain walls between them, a gatehouse in
/// the south wall, a keep with corner turrets and a hall in the bailey.
/// `half` is half the distance between opposite corner towers.
pub fn castle(ground: Ground, center: [f32; 2], half: f32, seed: u64) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let [cx, cz] = center;
    let corners = [
        [cx - half, cz - half],
        [cx + half, cz - half],
        [cx + half, cz + half],
        [cx - half, cz + half],
    ];
    let outside = |p: [f32; 2]| [cx + (p[0] - cx) * 3.0, cz + (p[1] - cz) * 3.0];
    for (i, c) in corners.iter().enumerate() {
        let tower = RoundTower {
            radius: 3.4,
            height: rng.range(12.0, 14.0),
            roof: i < 2,
            ruined: false,
        };
        s.merge(round_tower(
            ground,
            c[0],
            c[1],
            tower,
            seed ^ ((i as u64 + 1) * 977),
        ));
    }
    let thickness = 2.2;
    let height = 8.0;
    // North, east and west walls whole; the south one either side of the gate.
    for (k, (p, q)) in [(0, 1), (1, 2), (3, 0)].into_iter().enumerate() {
        let (p, q) = (corners[p], corners[q]);
        let mid = [(p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5];
        s.merge(curtain_wall(
            ground,
            p,
            q,
            thickness,
            height,
            outside(mid),
            seed ^ ((k as u64 + 11) * 7919),
        ));
    }
    let gate = 2.4;
    let gate_tower = 2.6;
    let south = cz + half;
    let edge = gate + gate_tower * 2.0;
    for (k, (p, q)) in [
        ([cx - half, south], [cx - edge, south]),
        ([cx + edge, south], [cx + half, south]),
    ]
    .into_iter()
    .enumerate()
    {
        s.merge(curtain_wall(
            ground,
            p,
            q,
            thickness,
            height,
            [cx, south + half],
            seed ^ ((k as u64 + 21) * 104_729),
        ));
    }
    for side in [-1.0f32, 1.0] {
        let x = cx + side * (gate + gate_tower);
        s.merge(square_tower(
            ground,
            x,
            south,
            gate_tower,
            height + 3.0,
            0.0,
            false,
            seed ^ 0x6a7e ^ side.to_bits() as u64,
        ));
    }
    // The gate's arch: a block over the passage, its opening dark below.
    let floor = ground(cx, south);
    let lintel = s.block(
        [cx, floor + 4.6, south],
        [gate, thickness * 0.5 + 0.4],
        height - 4.6 + 1.0,
        0.0,
        Material::Stone,
        0.95,
    );
    let mut gate_rng = Rng::new(seed ^ 0x9a7e);
    s.merlons(
        &lintel,
        [-gate, thickness * 0.5],
        [gate, thickness * 0.5],
        lintel.size[1],
        false,
        &mut gate_rng,
        0.95,
    );
    s.features.push(Feature {
        kind: FeatureKind::Gatehouse,
        base: [cx, floor, south],
        half: [edge, thickness],
        yaw: 0.0,
        height: height + 3.0,
        round: false,
    });

    // The keep, north of the middle, with a turret at each corner.
    let (kx, kz) = (cx - half * 0.2, cz - half * 0.3);
    let keep_half = 5.0;
    let keep_height = 21.0;
    let mut keep = square_tower(
        ground,
        kx,
        kz,
        keep_half,
        keep_height,
        0.0,
        false,
        seed ^ 0x6ee9,
    );
    keep.features[0].kind = FeatureKind::Keep;
    s.merge(keep);
    let keep_top = ground(kx, kz) + keep_height;
    for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let (x, z) = (kx + dx * keep_half, kz + dz * keep_half);
        let tint = rng.range(0.9, 1.05);
        s.drum([x, keep_top - 4.0, z], 1.15, 6.5, Material::Stone, tint);
        s.cone([x, keep_top + 2.5, z], 1.45, 2.8, Material::Slate, tint);
    }

    // A stone hall along the east wall.
    let hall = House {
        half_length: 6.5,
        half_width: 3.4,
        eaves: 6.0,
        roof: Material::Slate,
        stone: true,
    };
    let hx = cx + half - thickness * 0.5 - hall.half_width - 1.5;
    s.merge(house(
        ground,
        hx,
        cz + half * 0.25,
        std::f32::consts::FRAC_PI_2,
        hall,
        seed ^ 0x4a11,
    ));
    s
}

/// Houses along a road that winds away from `from` (a castle's gate)
/// towards `direction`, either side of it, kept clear of `avoid`.
pub fn village(
    ground: Ground,
    from: [f32; 2],
    direction: [f32; 2],
    houses: usize,
    avoid: &Structure,
    seed: u64,
) -> Structure {
    let mut rng = Rng::new(seed);
    let mut s = Structure::new();
    let d = normalize([direction[0], 0.0, direction[1]]);
    let across = [-d[2], d[0]];
    let mut along: f32 = 12.0;
    let mut placed = 0;
    let mut attempts = 0;
    while placed < houses && attempts < houses * 8 {
        attempts += 1;
        let side = if attempts % 2 == 0 { 1.0 } else { -1.0 };
        let wind = (along * 0.07).sin() * 5.0;
        let offset = side * rng.range(6.5, 8.5) + wind;
        let (x, z) = (
            from[0] + d[0] * along + across[0] * offset,
            from[1] + d[2] * along + across[1] * offset,
        );
        let spec = House {
            half_length: rng.range(3.0, 4.6),
            half_width: rng.range(2.3, 3.0),
            eaves: rng.range(4.6, 6.2),
            roof: [
                Material::Tile,
                Material::Thatch,
                Material::Tile,
                Material::Slate,
            ][(rng.next() * 4.0) as usize % 4],
            stone: false,
        };
        // Its front faces the road: its ridge runs along it.
        let yaw = (-d[2]).atan2(d[0])
            + if side > 0.0 {
                std::f32::consts::PI
            } else {
                0.0
            }
            + rng.range(-0.15, 0.15);
        let reach = (spec.half_length.powi(2) + spec.half_width.powi(2)).sqrt() + 2.0;
        let clear = |p: [f32; 3]| avoid.distance(p).min(s.distance(p)) > reach;
        if clear([x, ground(x, z) + 1.0, z]) {
            s.merge(house(
                ground,
                x,
                z,
                yaw,
                spec,
                seed ^ ((placed as u64 + 1) * 6151),
            ));
            placed += 1;
        }
        if side > 0.0 {
            along += rng.range(9.0, 12.0);
        }
    }
    s
}

/// A castle on its rise, the village below its gate, and a ruined
/// watchtower out to the west.
pub fn settlement(ground: Ground, seed: u64) -> Structure {
    let mut s = castle(ground, [0.0, -12.0], 15.0, seed);
    let houses = village(ground, [0.0, 3.0], [0.15, 1.0], 8, &s, seed ^ 0x7111);
    s.merge(houses);
    let ruin = RoundTower {
        radius: 2.8,
        height: 9.0,
        roof: false,
        ruined: true,
    };
    s.merge(round_tower(ground, -34.0, 8.0, ruin, seed ^ 0x2a1));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hill(x: f32, z: f32) -> f32 {
        3.0 * (-(x * x + (z + 12.0).powi(2)) / 900.0).exp()
            + 0.8 * (x * 0.05).sin() * (z * 0.04).cos()
    }

    #[test]
    fn every_face_is_wound_to_face_out() {
        let s = settlement(&hill, 3);
        let mesh = &s.mesh;
        assert!(
            mesh.triangle_count() > 2000,
            "{} triangles",
            mesh.triangle_count()
        );
        for tri in mesh.indices.chunks(3) {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| mesh.positions[i as usize]);
            let n = cross(sub(b, a), sub(c, a));
            let area = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if area < 1e-6 {
                continue;
            }
            let v = mesh.normals[tri[0] as usize];
            assert!(
                (n[0] * v[0] + n[1] * v[1] + n[2] * v[2]) / area > 0.0,
                "a face wound inwards at {a:?}"
            );
        }
    }

    #[test]
    fn buildings_stand_on_the_ground_and_reach_their_heights() {
        let ground = hill;
        let s = settlement(&ground, 3);
        assert!(
            s.features
                .iter()
                .filter(|f| f.kind == FeatureKind::House)
                .count()
                >= 5
        );
        // The gatehouse's passage is open at the foot.
        for feature in s
            .features
            .iter()
            .filter(|f| f.kind != FeatureKind::Gatehouse)
        {
            let [x, _, z] = feature.base;
            // Solid at the foot, all round: nothing floats on the slope.
            for p in feature.around(-0.3, 1.0) {
                let y = ground(p[0], p[2]);
                assert!(
                    s.distance([p[0], y + 0.1, p[2]]) < 0.0,
                    "{:?} floats at {p:?}",
                    feature.kind
                );
            }
            // And as tall as it was asked to be.
            let top = ground(x, z) + feature.height - 0.2;
            let at = feature.around(-0.4, 50.0)[0];
            assert!(
                s.distance([at[0], top, at[2]]) < 0.0,
                "{:?} short of {top}",
                feature.kind
            );
        }
    }

    #[test]
    fn the_castle_is_open_inside_its_walls() {
        let s = castle(&hill, [0.0, -12.0], 15.0, 3);
        // The bailey between the keep and the gate is open ground...
        let y = hill(4.0, -4.0);
        assert!(s.distance([4.0, y + 1.0, -4.0]) > 1.0);
        // ...and so is the passage under the gate.
        let g = hill(0.0, 3.0);
        assert!(s.distance([0.0, g + 1.5, 3.0]) > 0.5);
        assert!(s.distance([0.0, g + 6.0, 3.0]) < 0.0, "the lintel");
    }
}
