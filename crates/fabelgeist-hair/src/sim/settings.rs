//! Requested simulation detail and material/interaction settings.

use crate::parameters::{PointsPerStrand, StrandCount};
use fabelgeist_math::units::Metres;

#[derive(Clone, Copy, Debug)]
pub struct SimSettings {
    /// Strands simulated.
    pub guides: StrandCount,
    /// Points a simulated strand, resampled from the groom.
    pub points: PointsPerStrand,
    /// The hair fibre's Young's modulus, pascals: about 3.5 GPa dry.
    pub youngs_modulus: f32,
    /// The fibre's radius, metres: 20 to 50 micrometres.
    pub fibre_radius: f32,
    /// The fibre's density, kilograms a cubic metre.
    pub density: f32,
    /// How many fibres a simulated strand's clump holds together as one
    /// rod. Friction, oils and cohesion bind a lock's fibres, so it bends
    /// stiffer than the fibres would apart -- a bound bundle's stiffness
    /// grows as the square of its fibres, theirs apart as the count -- each
    /// fibre this many times a lone fibre's; and the air drags on the clump
    /// as one cylinder.
    pub clump: f32,
    /// The bending's viscous loss, seconds: its force per unit of the
    /// elastic force's rate.
    pub internal_damping: f32,
    /// The air's drag, as a share of a smooth cylinder's the clump's width:
    /// 0 in a vacuum.
    pub air_drag: f32,
    /// Metres a second squared.
    pub gravity: f32,
    /// The air's velocity, metres a second.
    pub wind: [f32; 3],
    /// How much the wind gusts, relative to its mean.
    pub turbulence: f32,
    /// Substeps a frame; on the GPU the strands meet each other at each.
    pub substeps: usize,
    /// The rods' implicit steps a substep.
    pub solver_substeps: usize,
    /// Metres kept off the head.
    pub collision_offset: Metres,
    /// Coulomb friction against the head and body.
    pub head_friction: f32,
    /// The groom is the rods' equilibrium under gravity, so it holds as
    /// groomed until moved; otherwise the rods rest as groomed, unloaded,
    /// and the hair sags under its weight when the simulation starts.
    pub sag_free: bool,
    /// How strongly hair denser than at rest is pushed apart, 0 to 1 a
    /// substep (GPU only).
    pub volume: f32,
    /// How much denser than at rest hair may get before it is, a share.
    pub volume_tolerance: f32,
    /// How far each point's velocity is drawn to the hair's around it, 0
    /// to 1 a substep (GPU only).
    pub friction: f32,
    /// Strands stop where they close on each other (GPU only).
    pub strand_collision: bool,
    /// The distance strands keep, metres: a simulated strand stands for a
    /// wisp this wide. 0 sizes it to the strands' spacing.
    pub collision_radius: f32,
    /// Coulomb friction where strands meet.
    pub collision_friction: f32,
    /// At most how many strands collide -- the first, an even sample of
    /// the head -- the rest meeting through the volume and friction alone:
    /// collisions cost more than the strands' count grows.
    pub collision_strands: StrandCount,
    /// Collide every this many substeps: hair swinging moves a good share
    /// of a wisp's width in a frame, and would pass through between checks
    /// a frame apart.
    pub collision_interval: usize,
    /// The density grid's cell, metres.
    pub grid_cell: f32,
}

/// How finely hair is simulated: how many strands, of how many points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimLevel {
    pub strands: StrandCount,
    pub points: PointsPerStrand,
}

impl SimSettings {
    /// These settings at `level`'s strands and points.
    pub fn at(self, level: SimLevel) -> Self {
        Self {
            guides: level.strands,
            points: level.points,
            ..self
        }
    }

    /// A fibre's mass a metre, kilograms.
    pub fn linear_density(&self) -> f32 {
        self.density * std::f32::consts::PI * self.fibre_radius * self.fibre_radius
    }

    /// A simulated strand's bending stiffness per fibre, newton square
    /// metres: a lone fibre's, `E π r⁴ / 4`, times the clump.
    pub fn bending_stiffness(&self) -> f32 {
        self.clump.max(1.0) * self.youngs_modulus * std::f32::consts::PI * self.fibre_radius.powi(4)
            / 4.0
    }

    /// Without the strands meeting each other: the solver alone, as the
    /// host's [`super::HairSim`] runs it.
    pub fn alone(self) -> Self {
        Self {
            volume: 0.0,
            friction: 0.0,
            strand_collision: false,
            ..self
        }
    }
}

impl SimLevel {
    pub const HIGH: Self = Self {
        strands: StrandCount::from_count(8192),
        points: PointsPerStrand::from_count(32),
    };
    pub const MEDIUM: Self = Self {
        strands: StrandCount::from_count(2048),
        points: PointsPerStrand::from_count(24),
    };
    pub const LOW: Self = Self {
        strands: StrandCount::from_count(512),
        points: PointsPerStrand::from_count(12),
    };
    /// Finest first.
    pub const ALL: [Self; 3] = [Self::HIGH, Self::MEDIUM, Self::LOW];

    pub fn name(&self) -> &'static str {
        match *self {
            Self::HIGH => "high",
            Self::MEDIUM => "medium",
            Self::LOW => "low",
            _ => "custom",
        }
    }
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            guides: StrandCount::from_count(2048),
            points: PointsPerStrand::from_count(24),
            youngs_modulus: 3.5e9,
            fibre_radius: 40e-6,
            density: 1300.0,
            clump: 50.0,
            internal_damping: 0.01,
            air_drag: 1.0,
            gravity: 9.81,
            wind: [0.0; 3],
            turbulence: 0.5,
            substeps: 4,
            solver_substeps: 4,
            collision_offset: Metres::from_metres(0.002),
            head_friction: 0.3,
            sag_free: true,
            volume: 0.35,
            volume_tolerance: 0.15,
            friction: 0.08,
            strand_collision: true,
            collision_radius: 0.0,
            collision_friction: 0.3,
            collision_strands: StrandCount::from_count(1024),
            collision_interval: 1,
            grid_cell: 0.01,
        }
    }
}

