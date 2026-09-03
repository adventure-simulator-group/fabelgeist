//! The measurements are checked against a body whose answers are known by
//! construction.
//!
//! A mannequin is built out of stacked rings with a radius profile chosen so
//! that its waist, hips, bust and underbust are at heights this file picked and
//! its girths are numbers this file can work out on its own -- the perimeter of
//! a regular polygon, which is `2 n r sin(pi/n)` and owes nothing to the code
//! being tested. If the measurer says the waist is 69.1 cm around at 105 cm off
//! the floor, that is because a ring of radius 11 was put there, not because
//! two halves of the same mistake agreed with each other.
//!
//! The mannequin is deliberately not a human being. Its shoulders are a cone
//! and its head is a ball, because what is being tested is that each level is
//! *found by the rule it claims to use* -- widest between here and there,
//! narrowest above that -- and a shape that makes those rules easy to check by
//! hand is a better test than one that looks like a person.

use super::*;
use crate::Design;

/// Sides per ring. Fine enough that a ring is round to within a millimetre,
/// coarse enough that a body is a few thousand triangles.
const SIDES: usize = 64;

/// The perimeter of a regular `SIDES`-gon inscribed in a circle. This is what
/// the hull of one of the mannequin's rings actually is, and the measurer
/// should return it rather than `2 pi r`.
fn ring_girth(radius: f32) -> f32 {
    2.0 * SIDES as f32 * radius * (std::f32::consts::PI / SIDES as f32).sin()
}

/// The torso's radius at a height, linearly between the keys below.
///
/// Reading upwards: the hips are widest at 86, the waist narrowest at 105,
/// there is a dip at 118 for the underbust to find, the bust is widest at 126,
/// the arms leave at 132, and above that a cone falls from the shoulders at
/// 133.5 to the neck at 136.5 -- a drop of 3 cm across 11 cm of width, which
/// is a shoulder inclination of `atan(3 / 11)`, 15.26 degrees.
const TORSO: &[(f32, f32)] = &[
    (82.0, 15.5),
    (86.0, 16.0),
    (96.0, 12.5),
    (105.0, 11.0),
    (112.0, 12.0),
    (118.0, 11.5),
    (126.0, 15.0),
    (132.0, 13.5),
    (133.5, 17.0),
    (136.5, 6.0),
];

const CROTCH: f32 = 82.0;
const HIPS_Y: f32 = 86.0;
const WAIST_Y: f32 = 105.0;
const BUST_Y: f32 = 126.0;
const ARMPIT_Y: f32 = 132.5;
const CROWN: f32 = 168.0;
const SHOULDER_X: f32 = 17.0;
const THIGH_R: f32 = 9.0;
const WRIST_R: f32 = 3.5;
/// Degrees below the horizontal that the arms are held at.
const ARM_POSE: f32 = 40.0;

struct Build {
    vertices: Vec<[f32; 3]>,
    faces: Vec<[u32; 3]>,
}

impl Build {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// A closed tube along a straight axis, from a stack of rings.
    ///
    /// `at` gives the centre and radius at a parameter running from 0 to 1
    /// along the axis; the tube is capped at both ends, so it is a closed
    /// surface and a plane cutting it gives a closed ring.
    fn tube(&mut self, steps: usize, at: impl Fn(f32) -> ([f32; 3], f32), normal: [f32; 3]) {
        let (u, v) = frame(normal);
        let first = self.vertices.len() as u32;

        for step in 0..=steps {
            let (centre, radius) = at(step as f32 / steps as f32);
            for side in 0..SIDES {
                // Half a step of offset, so no vertex lands exactly on an axis
                // and the two halves of a section are never ambiguous.
                let angle = (side as f32 + 0.5) * std::f32::consts::TAU / SIDES as f32;
                let (sin, cos) = angle.sin_cos();
                self.vertices.push([
                    centre[0] + radius * (cos * u[0] + sin * v[0]),
                    centre[1] + radius * (cos * u[1] + sin * v[1]),
                    centre[2] + radius * (cos * u[2] + sin * v[2]),
                ]);
            }
        }

        for step in 0..steps {
            for side in 0..SIDES {
                let next = (side + 1) % SIDES;
                let low = first + (step * SIDES + side) as u32;
                let low_next = first + (step * SIDES + next) as u32;
                let high = first + ((step + 1) * SIDES + side) as u32;
                let high_next = first + ((step + 1) * SIDES + next) as u32;
                self.faces.push([low, low_next, high_next]);
                self.faces.push([low, high_next, high]);
            }
        }

        for (end, ring) in [(0usize, 0usize), (1, steps)] {
            let (centre, _) = at(end as f32);
            let hub = self.vertices.len() as u32;
            self.vertices.push(centre);
            for side in 0..SIDES {
                let next = (side + 1) % SIDES;
                let a = first + (ring * SIDES + side) as u32;
                let b = first + (ring * SIDES + next) as u32;
                self.faces.push([hub, a, b]);
            }
        }
    }
}

/// Two axes perpendicular to `normal`.
fn frame(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let helper = if normal[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let dot = helper[0] * normal[0] + helper[1] * normal[1] + helper[2] * normal[2];
    let mut u = [
        helper[0] - normal[0] * dot,
        helper[1] - normal[1] * dot,
        helper[2] - normal[2] * dot,
    ];
    let length = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    u = [u[0] / length, u[1] / length, u[2] / length];
    let v = [
        u[1] * normal[2] - u[2] * normal[1],
        u[2] * normal[0] - u[0] * normal[2],
        u[0] * normal[1] - u[1] * normal[0],
    ];
    (u, v)
}

fn torso_radius(y: f32) -> f32 {
    if y <= TORSO[0].0 {
        return TORSO[0].1;
    }
    for pair in TORSO.windows(2) {
        let (low, high) = (pair[0], pair[1]);
        if y <= high.0 {
            let t = (y - low.0) / (high.0 - low.0);
            return low.1 + (high.1 - low.1) * t;
        }
    }
    TORSO[TORSO.len() - 1].1
}

/// The mannequin, and the joints a rig would have supplied with it.
fn mannequin() -> (Build, Landmarks) {
    let mut build = Build::new();
    let up = [0.0, 1.0, 0.0];

    // The torso, sampled finely enough that the shoulder cone has rings on it.
    let bottom = TORSO[0].0;
    let top = TORSO[TORSO.len() - 1].0;
    let steps = ((top - bottom) / 0.25).round() as usize;
    build.tube(
        steps,
        |t| {
            let y = bottom + (top - bottom) * t;
            ([0.0, y, 0.0], torso_radius(y))
        },
        up,
    );

    // Neck and head, so there is something above the shoulders to stop at.
    build.tube(8, |t| ([0.0, 136.5 + 15.5 * t, 0.0], 6.0), up);
    build.tube(
        24,
        |t| {
            let angle = std::f32::consts::PI * t;
            (
                [0.0, 159.0 - 9.0 * angle.cos(), 0.0],
                9.0 * angle.sin().max(0.02),
            )
        },
        up,
    );

    // Legs: plain cylinders, ending exactly where the torso begins, so that
    // the highest cut catching two of them is the crotch.
    for side in [1.0f32, -1.0] {
        build.tube(24, |t| ([side * 10.0, CROTCH * t, 0.0], THIGH_R), up);
    }

    // Arms: one straight tapered tube each, starting at the armpit. Below that
    // height a cut finds three rings, above it one, which is how the armpit is
    // found.
    let angle = ARM_POSE.to_radians();
    let direction = [angle.cos(), -angle.sin(), 0.0];
    let mut arms = Vec::new();
    for side in [1.0f32, -1.0] {
        let start = [side * SHOULDER_X, ARMPIT_Y, 0.0];
        let axis = [side * direction[0], direction[1], 0.0];
        build.tube(
            48,
            |t| {
                let along = 59.0 * t;
                // 6 cm at the shoulder, 5 at the elbow, 3.5 at the wrist, and
                // a little hand beyond it.
                let radius = match along {
                    a if a <= 26.0 => 6.0 - 1.0 * a / 26.0,
                    a if a <= 51.0 => 5.0 - 1.5 * (a - 26.0) / 25.0,
                    a => WRIST_R + 1.0 * (a - 51.0) / 8.0,
                };
                (
                    [
                        start[0] + axis[0] * along,
                        start[1] + axis[1] * along,
                        start[2] + axis[2] * along,
                    ],
                    radius,
                )
            },
            axis,
        );
        let point = |along: f32| {
            [
                start[0] + axis[0] * along,
                start[1] + axis[1] * along,
                start[2] + axis[2] * along,
            ]
        };
        arms.push((start, point(26.0), point(51.0)));
    }

    let landmarks = Landmarks {
        shoulders: [arms[0].0, arms[1].0],
        elbows: [arms[0].1, arms[1].1],
        wrists: [arms[0].2, arms[1].2],
        hips: [[10.0, 84.0, 0.0], [-10.0, 84.0, 0.0]],
        knees: [[10.0, 45.0, 0.0], [-10.0, 45.0, 0.0]],
        neck: [0.0, 140.0, 0.0],
    };
    (build, landmarks)
}

fn measured() -> Body {
    let (build, landmarks) = mannequin();
    let mesh = BodyMesh {
        vertices: &build.vertices,
        faces: &build.faces,
    };
    measure(mesh, &landmarks).expect("the mannequin should measure")
}

#[track_caller]
fn close(name: &str, got: f64, want: f32, tolerance: f32) {
    assert!(
        (got as f32 - want).abs() <= tolerance,
        "{name}: got {got:.3}, expected {want:.3} +/- {tolerance}"
    );
}

/// The girths are the perimeters of the rings that were put at those heights,
/// and each was found by the rule its level claims: widest, narrowest, dip.
#[test]
fn finds_the_girths_it_was_built_with() {
    let body = measured();

    close("height", body.get("height"), CROWN, 0.2);
    close("hips", body.get("hips"), ring_girth(16.0), 0.6);
    close("waist", body.get("waist"), ring_girth(11.0), 0.6);
    close("bust", body.get("bust"), ring_girth(15.0), 0.6);
    close("underbust", body.get("underbust"), ring_girth(11.5), 0.6);
    close("leg_circ", body.get("leg_circ"), ring_girth(THIGH_R), 0.6);
    close("wrist", body.get("wrist"), ring_girth(WRIST_R), 0.6);
}

/// The heights come back as differences between levels, so getting them right
/// means every level was found in the right place and not merely a plausible
/// one.
#[test]
fn finds_the_levels_it_was_built_with() {
    let body = measured();

    // Waist to hips, and hips to crotch.
    close("hips_line", body.get("hips_line"), WAIST_Y - HIPS_Y, 0.6);
    close(
        "crotch_hip_diff",
        body.get("crotch_hip_diff"),
        HIPS_Y - CROTCH,
        0.8,
    );

    // The nape sits at the neck base, and the bust line is measured down from
    // it -- so this pins the neck base and the bust level together.
    let neck_base = body.get("height") - body.get("head_l");
    close("vert_bust_line", body.get("vert_bust_line"), 0.0, 100.0);
    assert!(
        body.get("vert_bust_line") > 0.0,
        "the bust must come out below the nape"
    );
    assert!(
        (neck_base - BUST_Y as f64 - body.get("vert_bust_line")).abs() < 6.0,
        "nape at {neck_base:.1}, bust at {BUST_Y}, but vert_bust_line is {:.1}",
        body.get("vert_bust_line")
    );
}

/// The angles, and the measurements taken off the joints rather than the skin.
#[test]
fn measures_the_pose_and_the_frame() {
    let body = measured();

    close("shoulder_w", body.get("shoulder_w"), 2.0 * SHOULDER_X, 0.01);
    close("arm_length", body.get("arm_length"), 51.0, 0.01);
    close("arm_pose_angle", body.get("arm_pose_angle"), ARM_POSE, 0.01);

    // The shoulder cone falls 3 cm across 11 cm of width, whichever part of it
    // is looked at.
    close(
        "shoulder_incl",
        body.get("shoulder_incl"),
        (3.0f32 / 11.0).atan().to_degrees(),
        1.5,
    );

    // The side runs out from an 11 cm waist to a 16 cm hip over 19 cm of drop.
    close(
        "hip_inclination",
        body.get("hip_inclination"),
        (5.0f32 / (WAIST_Y - HIPS_Y)).atan().to_degrees(),
        1.5,
    );
}

/// Every measurement a garment program reads has to be there, and be a number
/// a body could have. A missing one panics inside `Body::get`, which is how a
/// pattern would fail -- better to fail here.
#[test]
fn fills_in_every_measurement_a_pattern_asks_for() {
    let body = measured();
    let reference = Body::from_yaml_str(crate::assets::BODIES[0].yaml).unwrap();

    for name in reference.names().collect::<Vec<_>>() {
        assert!(
            body.contains(name),
            "the measurer left out '{name}', which the bundled bodies have"
        );
        let value = body.get(name);
        assert!(value.is_finite(), "'{name}' came out {value}");
        assert!(
            value >= 0.0,
            "'{name}' came out negative ({value}); no measurement is"
        );
    }
}

/// The measured body has to drive the pattern library, not just look like a
/// body: the derived measurements are what the panels are placed by.
#[test]
fn drives_the_pattern_library() {
    let body = measured();

    // `_waist_level` is where every waistband and skirt is placed, and it is
    // derived rather than measured -- height less the head less the tape down
    // the back. It has to land on the waist the body actually has.
    close("_waist_level", body.get("_waist_level"), WAIST_Y, 6.0);
    assert!(body.get("_leg_length") > 0.0);

    let design = Design::from_yaml_str(crate::assets::DESIGNS[0].yaml).unwrap();
    let garment = crate::programs::MetaGarment::new("measured", &body, &design);
    let spec = garment.assembly();
    assert!(
        !spec.panels.is_empty(),
        "the pattern came out with no panels"
    );
}

/// A mesh in metres is the mistake this will actually be made with, and it is
/// caught rather than quietly producing a doll's measurements.
#[test]
fn rejects_a_body_in_the_wrong_unit() {
    let (build, landmarks) = mannequin();
    let metres: Vec<[f32; 3]> = build
        .vertices
        .iter()
        .map(|v| [v[0] / 100.0, v[1] / 100.0, v[2] / 100.0])
        .collect();
    let error = measure(
        BodyMesh {
            vertices: &metres,
            faces: &build.faces,
        },
        &landmarks,
    )
    .expect_err("a body 1.7 units tall is not in centimetres");
    assert!(format!("{error}").contains("centimetres"), "{error}");
}

#[test]
fn rejects_an_empty_mesh() {
    let landmarks = mannequin().1;
    assert!(
        measure(
            BodyMesh {
                vertices: &[],
                faces: &[]
            },
            &landmarks
        )
        .is_err()
    );
}

#[test]
fn shoulder_slope_does_not_include_the_head_above_a_narrow_neck() {
    let (build, landmarks) = mannequin();
    let measurer = Measurer::new(
        BodyMesh {
            vertices: &build.vertices,
            faces: &build.faces,
        },
        &landmarks,
    )
    .unwrap();
    close(
        "shoulder slope",
        measurer.shoulder_inclination(12.0, 34.0, ARMPIT_Y) as f64,
        (3.0f32 / 11.0).atan().to_degrees(),
        1.5,
    );
}
