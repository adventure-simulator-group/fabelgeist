//! Plant types: the structural (Sec. 5, Table 4) and plasticity (Sec. 6,
//! Tables 1 and 3) parameters of a species.

use crate::units::{Celsius, CelsiusDelta, Millimetres};
pub use catalog::Table4Row;

mod catalog;

/// How a species' foliage is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foliage {
    Broadleaf = 0,
    Needles = 1,
    Fronds = 2,
}

/// Whether foliage contributes canopy litter to the ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LitterContribution {
    Canopy,
    GroundCover,
}

/// Everything that distinguishes one species from another.
/// Field names follow the paper's symbols where it has them.
#[derive(Clone, Debug, PartialEq)]
pub struct PlantType {
    pub name: String,
    pub foliage: Foliage,
    pub litter: LitterContribution,

    // --- Structural and physiological (Table 4) -------------------------
    /// `p_max`: age in years after which the plant senesces.
    pub max_age: f32,
    /// `v_root_max`: the most vigor the root module is ever allocated.
    pub root_vigor_max: f32,
    /// `alpha` of the Borchert-Honda model the paper extends (Palubicki et
    /// al. 2009): the root receives `alpha Q_total`, capped at `v_root_max`.
    /// It sets how much vigor a young plant has to spare -- enough for
    /// laterals to survive strong apical control -- while the cap, divided
    /// by `v_min`, bounds how many modules an old one can hold.
    pub vigor_scale: f32,
    /// `g_p`: growth rate of the whole plant, physiological age per year at
    /// full vigor.
    pub growth_rate: f32,
    /// `lambda`: apical control; above 0.5 excurrent, below decurrent.
    pub apical_control: f32,
    /// `lambda_mature`: apical control once flowering.
    pub apical_control_mature: f32,
    /// `D`: determinacy; high values monopodial, low sympodial.
    pub determinacy: f32,
    /// `D_mature`: determinacy once flowering.
    pub determinacy_mature: f32,
    /// `F_age`: flowering age in years, scaled per plant by its vigor.
    pub flowering_age: f32,
    /// `alpha`: preferred angle of a lateral module's axis from the
    /// vertical, radians. Leaders aim straight up.
    pub tropism_angle: f32,
    /// `w_2`: weight of the tropism term when orienting modules.
    pub tropism_weight: f32,
    /// `w_1`: weight of the collision term when orienting modules.
    pub collision_weight: f32,
    /// `g_2`: strength of the tropism offset on segments; negative droops
    /// (gravitropism), positive rises (phototropism).
    pub tropism_strength: f32,
    /// `g_1`: how quickly the tropism offset fades as a segment ages.
    pub tropism_decay: f32,
    /// `phi`: diameter of a branch tip, metres.
    pub tip_diameter: f32,
    /// `beta`: segment length per unit of physiological age, metres.
    pub length_scale: f32,
    /// `l_max`: the longest a segment gets, metres.
    pub segment_length_max: f32,
    /// `v_min`: modules below this vigor are shed, and terminals below it
    /// stay empty.
    pub vigor_min: f32,
    /// `v_max`: vigor at which a module develops at the full growth rate.
    pub vigor_max: f32,
    /// Years over which `v_root_max` falls to zero once `max_age` is passed.
    pub senescence_years: f32,

    // --- Plasticity (Tables 1 and 3) -------------------------------------
    /// Seeds per seeding period (a year).
    pub seeding_frequency: f32,
    /// Standard deviation of seed placement around the parent, metres.
    pub seeding_radius: f32,
    /// `s_tol` in `[0, 1]`: how well the plant grows in shade.
    pub shade_tolerance: f32,
    /// `T_A`: optimal habitat temperature, degrees Celsius.
    pub temperature_optimum: Celsius,
    /// Spread of the temperature adaptation function.
    pub temperature_tolerance: CelsiusDelta,
    /// `P_A`: optimal habitat precipitation, mm per year.
    pub precipitation_optimum: Millimetres,
    /// Spread of the precipitation adaptation function.
    pub precipitation_tolerance: Millimetres,

    // --- Appearance ------------------------------------------------------
    pub bark_color: [f32; 3],
    pub leaf_color: [f32; 3],
    /// Edge of a leaf card, metres; zero for leafless.
    pub leaf_size: f32,
    /// Years a module keeps its leaves, or `None` for as long as it lives.
    /// A palm's fronds grow only at the crown because the trunk's old ones
    /// fall; branching trees shed their inner foliage by shading instead.
    pub leaf_years: Option<f32>,

    // --- Self-pruning and mechanics (not in the paper) -------------------
    /// The light a branch must average over everything it carries to stay
    /// on the tree: its compensation point, as a fraction of open sky
    /// (`Q Q_G`). Branches are autonomous in their carbon -- the trunk does
    /// not feed a branch that cannot feed itself -- so a branch below this
    /// is shed however much vigor the plant has. Shade-tolerant species
    /// have lower compensation points.
    pub light_compensation: f32,
    /// Green wood density, kg/m^3.
    pub wood_density: f32,
    /// Young's modulus of green wood, Pa: how much a loaded branch sags.
    pub wood_stiffness: f32,
    /// Modulus of rupture of green wood, Pa: the bending stress at which a
    /// branch snaps.
    pub wood_strength: f32,
    /// Mass of one leaf cluster, kg.
    pub leaf_mass: f32,
    /// How far below its strength a branch keeps the stress of its own
    /// weight. Wind and snow need the rest, so a branch nearing
    /// `wood_strength / safety_factor` stops lengthening -- growth goes to
    /// the rest of the crown -- rather than growing on until it snaps. It is
    /// what keeps limbs from reaching out further than wood can hold.
    pub safety_factor: f32,
    /// The tallest the species grows, metres: where water pulled up the stem
    /// no longer has the pressure to expand new shoots. Growth slows over
    /// the top 40% of it and stops at it.
    pub max_height: f32,
    /// Share of the sapwood's cross-section that stops conducting each year
    /// and joins the heartwood, so that stems keep thickening ring by ring
    /// after the crown stops growing. Zero for plants without secondary
    /// growth -- palms, grasses -- whose stems stay as thick as their
    /// crowns need.
    pub heartwood_rate: f32,
    /// How fast a branch grows against the trunk it grew from. The leader
    /// holding back its branches' elongation -- apical control in the
    /// botanical sense -- is what keeps an excurrent conifer narrow.
    /// `lambda` shares out vigor between buds, but a shoot with more vigor
    /// than `v_max` grows no faster, so it cannot do this alone.
    pub lateral_extension: f32,
    /// Years over which a branch's growth falls by half. A branch's shoots
    /// get shorter as it ages -- more and more shaded by the crown growing
    /// above it, further from the roots, its meristems older -- so a
    /// conifer's lowest limbs level off at a few metres while the trunk
    /// climbs on. The light model barely darkens the inside of a crown, so
    /// this carries that decline. Infinite for branches that keep their
    /// vigor, as a decurrent crown's main stems do.
    pub branch_half_life: f32,
    /// How much harder water moves along a metre of branch than up a metre
    /// of trunk: branch junctions constrict the xylem and branch vessels are
    /// narrower (Zimmermann's hydraulic segmentation). Friction takes about
    /// half the water's pull at the ceiling, gravity the rest, so a shoot's
    /// hydraulic distance from the roots is its height plus half this times
    /// the branch between it and the trunk; [`PlantType::max_height`] bounds
    /// that distance, so limbs, like trunks, stop lengthening when water no
    /// longer reaches their tips.
    pub branch_resistance: f32,
    /// The share of its height below which the trunk keeps no branches:
    /// buds there stay dormant, and branches the rising crown leaves behind
    /// are shed. A maturing baobab clears its swollen trunk this way, its
    /// crown gathered at the top; zero keeps branches down to the ground.
    pub clear_bole: f32,
    /// New stems a year a plant sends up from around its base once
    /// established -- how a shrub renews itself as its older stems die.
    /// Each is a plant of its own, leaning out of the clump.
    pub sprouts_per_year: f32,
    /// The most stems a clump holds: a stem sprouts only while fewer stand
    /// around it.
    pub clump_stems: u32,
    /// Never branches: every module is Corner's model, one axis, its leaves
    /// all from the growing tip -- a palm. Without a cambium its stem stays
    /// as thick as the tip made it, so `tip_diameter` is the stem's.
    pub unbranched: bool,
}

impl Default for PlantType {
    fn default() -> Self {
        Self {
            name: "plant".into(),
            foliage: Foliage::Broadleaf,
            litter: LitterContribution::Canopy,
            max_age: 300.0,
            root_vigor_max: 400.0,
            vigor_scale: 4.0,
            growth_rate: 0.25,
            apical_control: 0.8,
            apical_control_mature: 0.8,
            determinacy: 0.8,
            determinacy_mature: 0.8,
            flowering_age: 40.0,
            tropism_angle: 1.2,
            tropism_weight: 0.5,
            collision_weight: 1.0,
            tropism_strength: 0.2,
            tropism_decay: 1.0,
            tip_diameter: 0.015,
            length_scale: 1.0,
            segment_length_max: 0.5,
            vigor_min: 0.3,
            vigor_max: 2.0,
            senescence_years: 20.0,
            seeding_frequency: 3.0,
            seeding_radius: 5.0,
            shade_tolerance: 0.5,
            temperature_optimum: Celsius::from_degrees(15.0),
            temperature_tolerance: CelsiusDelta::from_degrees(8.0),
            precipitation_optimum: Millimetres::from_millimetres(1000.0),
            precipitation_tolerance: Millimetres::from_millimetres(800.0),
            bark_color: [0.36, 0.27, 0.2],
            leaf_color: [0.25, 0.45, 0.16],
            leaf_size: 0.35,
            leaf_years: None,
            light_compensation: compensation(0.5),
            wood_density: 600.0,
            wood_stiffness: 9.0e9,
            wood_strength: 4.5e7,
            leaf_mass: 0.05,
            safety_factor: 4.0,
            max_height: 30.0,
            heartwood_rate: 0.01,
            lateral_extension: 1.0,
            branch_resistance: 3.0,
            branch_half_life: f32::INFINITY,
            clear_bole: 0.0,
            sprouts_per_year: 0.0,
            clump_stems: 1,
            unbranched: false,
        }
    }
}

/// A compensation point for a shade tolerance: from a quarter of open sky
/// for an intolerant plant down to a twentieth for a fully tolerant one.
pub fn compensation(shade_tolerance: f32) -> f32 {
    0.05 + 0.2 * (1.0 - shade_tolerance.clamp(0.0, 1.0))
}

impl PlantType {
    /// Plasticity parameters from Table 3, which the paper gives as seeding
    /// frequency, seeding radius, shade tolerance, `T_A` and `P_A`.
    pub fn with_plasticity(
        mut self,
        seeding_frequency: f32,
        seeding_radius: f32,
        shade_tolerance: f32,
        temperature_optimum: Celsius,
        precipitation_optimum: Millimetres,
    ) -> Self {
        self.seeding_frequency = seeding_frequency;
        self.seeding_radius = seeding_radius;
        self.shade_tolerance = shade_tolerance;
        self.light_compensation = compensation(shade_tolerance);
        self.temperature_optimum = temperature_optimum;
        self.precipitation_optimum = precipitation_optimum;
        self
    }

    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Fig. 19's type (I): a fast, cold-adapted shrub.
    pub fn shrub() -> Self {
        Self {
            leaf_color: [0.34, 0.46, 0.18],
            bark_color: [0.30, 0.24, 0.2],
            max_height: 3.0,
            // A clump of stems, each living a couple of decades, renewed
            // from the base; a sprout from an established root system grows
            // fast. It spreads by sprouting more than by seed, so it seeds
            // less than Fig. 19's five a year.
            growth_rate: 0.35,
            sprouts_per_year: 0.4,
            clump_stems: 4,
            ..Self::table4(Table4Row::A)
        }
        .with_plasticity(
            1.0,
            8.0,
            0.15,
            Celsius::from_degrees(8.0),
            Millimetres::from_millimetres(672.0),
        )
        .named("shrub")
    }

    /// Fig. 19's type (II): a conifer.
    pub fn conifer() -> Self {
        Self {
            foliage: Foliage::Needles,
            leaf_color: [0.11, 0.28, 0.16],
            bark_color: [0.33, 0.22, 0.16],
            leaf_size: 0.4,
            max_height: 40.0,
            heartwood_rate: 0.02,
            // A narrow cone: branches grow at a third of the leader's pace,
            // slowing as they age, and get a little less vigor than Table
            // 4's row f gives.
            apical_control: 0.8,
            apical_control_mature: 0.8,
            lateral_extension: 0.35,
            branch_half_life: 30.0,
            ..Self::table4(Table4Row::F)
        }
        .with_plasticity(
            2.0,
            3.0,
            0.45,
            Celsius::from_degrees(12.0),
            Millimetres::from_millimetres(672.0),
        )
        .named("conifer")
    }

    /// Fig. 19's type (III): a broadleaf deciduous tree.
    pub fn deciduous() -> Self {
        Self {
            leaf_color: [0.28, 0.5, 0.14],
            bark_color: [0.4, 0.34, 0.28],
            leaf_size: 0.45,
            max_height: 35.0,
            ..Self::table4(Table4Row::G)
        }
        .with_plasticity(
            1.0,
            3.0,
            0.6,
            Celsius::from_degrees(16.0),
            Millimetres::from_millimetres(672.0),
        )
        .named("deciduous")
    }

    /// A grass or forb: tiny, short-lived, many seeds.
    pub fn grass() -> Self {
        Self {
            litter: LitterContribution::GroundCover,
            leaf_color: [0.45, 0.55, 0.2],
            segment_length_max: 0.15,
            leaf_size: 0.12,
            max_height: 0.8,
            heartwood_rate: 0.0,
            ..Self::table4(Table4Row::D)
        }
        .with_plasticity(
            8.0,
            4.0,
            0.2,
            Celsius::from_degrees(20.0),
            Millimetres::from_millimetres(600.0),
        )
        .named("grass")
    }

    /// Fig. 6: a baobab, whose crown changes when it flowers.
    pub fn baobab() -> Self {
        Self {
            bark_color: [0.5, 0.45, 0.4],
            leaf_color: [0.35, 0.5, 0.2],
            max_height: 20.0,
            // A swollen, water-storing trunk, bare below a broad crown of
            // thick limbs gathered at the top. The limbs store water too, so
            // their length costs little of it. Table 4's row keeps a young
            // tree an unbranched pole (lambda 1) until it flowers at 58; a
            // young baobab branches like any sapling, and flowers at twenty
            // to thirty.
            heartwood_rate: 0.04,
            clear_bole: 0.5,
            branch_resistance: 1.0,
            apical_control: 0.9,
            flowering_age: 25.0,
            ..Self::table4(Table4Row::E)
        }
        .with_plasticity(
            1.0,
            6.0,
            0.2,
            Celsius::from_degrees(27.0),
            Millimetres::from_millimetres(600.0),
        )
        .named("baobab")
    }

    /// A palm-like single-trunk tree.
    pub fn palm() -> Self {
        Self {
            foliage: Foliage::Fronds,
            leaf_color: [0.3, 0.55, 0.2],
            bark_color: [0.47, 0.42, 0.35],
            // One unbranched stem about 30 cm thick, crowned by fronds
            // four to five metres long that live about three years, climbing
            // half a metre a year rather than Table 4's row c's metre.
            unbranched: true,
            growth_rate: 0.45,
            tip_diameter: 0.3,
            leaf_size: 1.1,
            leaf_years: Some(3.0),
            segment_length_max: 0.6,
            max_height: 25.0,
            heartwood_rate: 0.0,
            ..Self::table4(Table4Row::C)
        }
        .with_plasticity(
            3.0,
            6.0,
            0.3,
            Celsius::from_degrees(26.0),
            Millimetres::from_millimetres(2500.0),
        )
        .named("palm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table4_has_sixteen_rows() {
        assert_eq!(PlantType::table4_all().len(), 16);
        assert!(Table4Row::try_from('z').is_err());
        let baobab = PlantType::table4(Table4Row::E);
        assert_eq!(baobab.apical_control_mature, 0.5);
        assert_eq!(baobab.determinacy_mature, 0.33);
        assert_eq!(baobab.flowering_age, 58.0);
    }
}
