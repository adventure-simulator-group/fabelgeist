//! What a fabric is, as far as the solver is concerned.
//!
//! Compliance rather than stiffness, because that is what XPBD takes: it is
//! the inverse, in metres per newton, and zero means inextensible. The numbers
//! below are not measured textile data -- they are the values that make each
//! fabric read as itself when a garment is draped, which is what a fitting
//! tool actually needs.

/// Material parameters for one garment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fabric {
    /// Areal density, kg per square metre. Sets the particle masses, and with
    /// them how the cloth hangs.
    pub density: f32,
    /// Resistance to stretching along the mesh edges. Near zero for anything
    /// woven: real cloth barely stretches in the warp and weft.
    pub stretch_compliance: f32,
    /// Resistance to bending at the hinge between two triangles. This is the
    /// parameter that decides whether the fabric falls in a few heavy folds or
    /// many light ones, and it is the one worth exposing to a user.
    ///
    /// The useful range is roughly `1e-6` (a board) to `1e-3` (a rag); the
    /// values below were picked by draping a flap and measuring how far it
    /// reaches. Resolution-independent, because the bending weights carry the
    /// mesh's length scale -- see `topology::bending_weights`.
    pub bend_compliance: f32,
    /// Resistance at a seam. Stiffer than the fabric itself, because a sewn
    /// seam is two layers plus thread.
    pub seam_compliance: f32,
    /// Fabric thickness. Half of it is the particle radius used for collision
    /// against the body and against the cloth itself.
    pub thickness: f32,
    /// Coulomb friction against the body.
    pub friction: f32,
    /// Velocity drag, per second. Air resistance, in effect: it is what stops
    /// a draped garment swinging forever.
    pub damping: f32,
}

impl Default for Fabric {
    fn default() -> Self {
        Self::COTTON
    }
}

impl Fabric {
    /// A plain medium-weight woven: the default for anything unspecified.
    pub const COTTON: Self = Self {
        density: 0.20,
        stretch_compliance: 1e-7,
        bend_compliance: 3e-5,
        seam_compliance: 1e-8,
        thickness: 0.0006,
        friction: 0.35,
        damping: 0.6,
    };

    /// Light, slippery, and it falls in many fine folds.
    pub const SILK: Self = Self {
        density: 0.08,
        stretch_compliance: 1e-7,
        bend_compliance: 3e-4,
        seam_compliance: 1e-8,
        thickness: 0.0003,
        friction: 0.12,
        damping: 0.4,
    };

    /// Heavy and stiff: few folds, and they hold their shape.
    pub const DENIM: Self = Self {
        density: 0.45,
        stretch_compliance: 5e-8,
        bend_compliance: 4e-6,
        seam_compliance: 1e-9,
        thickness: 0.0012,
        friction: 0.5,
        damping: 0.9,
    };

    /// Heavy but soft, and it clings.
    pub const WOOL: Self = Self {
        density: 0.30,
        stretch_compliance: 2e-7,
        bend_compliance: 8e-5,
        seam_compliance: 1e-8,
        thickness: 0.0010,
        friction: 0.55,
        damping: 0.8,
    };

    /// Knitted: it stretches, which is the whole point of it.
    pub const JERSEY: Self = Self {
        density: 0.18,
        stretch_compliance: 5e-6,
        bend_compliance: 1.5e-4,
        seam_compliance: 1e-7,
        thickness: 0.0008,
        friction: 0.4,
        damping: 0.6,
    };

    pub const PRESETS: &'static [(&'static str, Self)] = &[
        ("Cotton", Self::COTTON),
        ("Silk", Self::SILK),
        ("Denim", Self::DENIM),
        ("Wool", Self::WOOL),
        ("Jersey", Self::JERSEY),
    ];

    /// Half the thickness: how far a particle's centre is held off a surface.
    pub fn particle_radius(&self) -> f32 {
        self.thickness * 0.5
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn with_bend_compliance(mut self, compliance: f32) -> Self {
        self.bend_compliance = compliance;
        self
    }

    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }
}
