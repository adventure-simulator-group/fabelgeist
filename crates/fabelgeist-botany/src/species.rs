//! Plant types: the structural (Sec. 5, Table 4) and plasticity (Sec. 6,
//! Tables 1 and 3) parameters of a species.

/// Everything that distinguishes one species from another.
///
/// Field names follow the paper's symbols where it has them.
#[derive(Clone, Debug, PartialEq)]
pub struct PlantType {
    pub name: String,

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
    pub temperature_optimum: f32,
    /// Spread of the temperature adaptation function.
    pub temperature_tolerance: f32,
    /// `P_A`: optimal habitat precipitation, mm per year.
    pub precipitation_optimum: f32,
    /// Spread of the precipitation adaptation function.
    pub precipitation_tolerance: f32,

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
            temperature_optimum: 15.0,
            temperature_tolerance: 8.0,
            precipitation_optimum: 1000.0,
            precipitation_tolerance: 800.0,
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

/// One row of the paper's Table 4.
struct Row {
    max_age: f32,
    root_vigor_max: f32,
    growth_rate: f32,
    lambda: (f32, Option<f32>),
    d: (f32, Option<f32>),
    flowering_age: f32,
    alpha: f32,
    w2: f32,
    g2: f32,
    phi_cm: f32,
    beta: f32,
}

/// Table 4, rows a..p.
///
/// The printed table's last five columns are headed `alpha, w2, g1, phi,
/// beta`. Read literally they do not all make sense, so:
///
/// * `alpha` is read as `cos(alpha)`. Its values run from -0.27 to 1, and
///   rows j and k differ only in it (0.45 against -0.2) -- an upright and a
///   weeping form of one tree. As a cosine that is exactly the difference.
/// * The third-from-last column carries the negative values the text gives
///   to `g2`, the signed tropism strength, so it is read as `g2`; `g1` keeps
///   its default.
/// * `phi` is read in centimetres -- five metres is not a twig.
const TABLE4: [(char, Row); 16] = {
    #[allow(clippy::too_many_arguments)]
    const fn row(
        max_age: f32,
        root_vigor_max: f32,
        growth_rate: f32,
        lambda: (f32, Option<f32>),
        d: (f32, Option<f32>),
        flowering_age: f32,
        alpha: f32,
        w2: f32,
        g2: f32,
        phi_cm: f32,
        beta: f32,
    ) -> Row {
        Row {
            max_age,
            root_vigor_max,
            growth_rate,
            lambda,
            d,
            flowering_age,
            alpha,
            w2,
            g2,
            phi_cm,
            beta,
        }
    }
    [
        (
            'a',
            row(
                20.0,
                42.0,
                0.19,
                (0.62, None),
                (0.25, None),
                0.0,
                0.52,
                0.63,
                -0.38,
                0.57,
                0.47,
            ),
        ),
        (
            'b',
            row(
                200.0,
                78.0,
                0.30,
                (0.84, None),
                (1.0, None),
                0.0,
                0.52,
                0.63,
                -1.2,
                1.00,
                0.79,
            ),
        ),
        (
            'c',
            row(
                80.0,
                11.0,
                0.80,
                (1.0, None),
                (1.0, None),
                0.0,
                0.9,
                0.5,
                1.0,
                5.00,
                1.95,
            ),
        ),
        (
            'd',
            row(
                16.0,
                1.7,
                0.23,
                (0.44, None),
                (0.31, None),
                0.0,
                1.0,
                1.0,
                1.0,
                5.00,
                3.00,
            ),
        ),
        (
            'e',
            row(
                430.0,
                600.0,
                0.15,
                (1.0, Some(0.5)),
                (1.0, Some(0.33)),
                58.0,
                0.52,
                0.63,
                0.56,
                3.00,
                1.23,
            ),
        ),
        (
            'f',
            row(
                550.0,
                450.0,
                0.20,
                (0.76, None),
                (0.82, None),
                0.0,
                0.17,
                0.5,
                0.47,
                1.20,
                1.90,
            ),
        ),
        (
            'g',
            row(
                550.0,
                700.0,
                0.20,
                (0.9, Some(0.5)),
                (0.93, Some(0.74)),
                55.0,
                0.17,
                0.5,
                0.47,
                1.38,
                0.94,
            ),
        ),
        (
            'h',
            row(
                500.0,
                570.0,
                0.24,
                (1.0, Some(0.5)),
                (1.0, Some(0.5)),
                55.0,
                0.5,
                0.27,
                -0.66,
                1.38,
                1.29,
            ),
        ),
        (
            'i',
            row(
                950.0,
                900.0,
                0.12,
                (0.87, Some(0.34)),
                (0.93, Some(0.55)),
                57.0,
                0.66,
                0.14,
                0.2,
                1.41,
                1.29,
            ),
        ),
        (
            'j',
            row(
                950.0,
                600.0,
                0.14,
                (1.0, Some(0.5)),
                (1.0, Some(0.51)),
                57.0,
                0.45,
                0.63,
                -0.9,
                0.82,
                0.93,
            ),
        ),
        (
            'k',
            row(
                950.0,
                600.0,
                0.14,
                (1.0, Some(0.5)),
                (1.0, Some(0.51)),
                57.0,
                -0.2,
                0.63,
                -0.9,
                0.82,
                0.93,
            ),
        ),
        (
            'l',
            row(
                1000.0,
                815.0,
                0.19,
                (0.92, Some(0.7)),
                (0.59, Some(0.56)),
                80.0,
                -0.19,
                0.72,
                -0.21,
                5.00,
                1.54,
            ),
        ),
        (
            'm',
            row(
                130.0,
                400.0,
                0.21,
                (0.88, Some(0.43)),
                (0.9, Some(0.7)),
                66.0,
                0.85,
                0.55,
                0.9,
                1.42,
                1.11,
            ),
        ),
        (
            'n',
            row(
                52.0,
                200.0,
                0.55,
                (0.96, Some(0.43)),
                (0.48, Some(0.7)),
                0.0,
                -0.27,
                0.43,
                0.73,
                1.50,
                2.50,
            ),
        ),
        (
            'o',
            row(
                300.0,
                600.0,
                0.20,
                (0.8, None),
                (0.86, None),
                0.0,
                -0.19,
                0.81,
                1.0,
                1.00,
                1.60,
            ),
        ),
        (
            'p',
            row(
                450.0,
                450.0,
                0.15,
                (1.0, Some(0.5)),
                (0.66, Some(0.33)),
                135.0,
                0.52,
                0.32,
                0.42,
                1.50,
                1.06,
            ),
        ),
    ]
};

impl PlantType {
    /// The structural parameters of Table 4, row `letter` (`'a'..='p'`),
    /// with default plasticity.
    ///
    /// A flowering age of zero in the table means the architecture does not
    /// change at maturity; the plant still has to flower to seed, so it is
    /// given one at a fifth of its lifespan.
    pub fn table4(letter: char) -> Option<Self> {
        let (_, row) = TABLE4
            .iter()
            .find(|(l, _)| *l == letter.to_ascii_lowercase())?;
        let small = row.root_vigor_max < 50.0;
        Some(Self {
            name: format!("table4-{letter}"),
            max_age: row.max_age,
            root_vigor_max: row.root_vigor_max,
            growth_rate: row.growth_rate,
            apical_control: row.lambda.0,
            apical_control_mature: row.lambda.1.unwrap_or(row.lambda.0),
            determinacy: row.d.0,
            determinacy_mature: row.d.1.unwrap_or(row.d.0),
            flowering_age: if row.flowering_age > 0.0 {
                row.flowering_age
            } else {
                row.max_age * 0.2
            },
            tropism_angle: row.alpha.clamp(-1.0, 1.0).acos(),
            tropism_weight: row.w2,
            tropism_strength: row.g2 * 0.25,
            tip_diameter: row.phi_cm * 0.01,
            length_scale: row.beta,
            segment_length_max: if small { 0.25 } else { 0.5 },
            senescence_years: (row.max_age * 0.1).max(3.0),
            leaf_size: if small { 0.15 } else { 0.35 },
            max_height: if small { 4.0 } else { 30.0 },
            ..Self::default()
        })
    }

    /// Every row of Table 4.
    pub fn table4_all() -> Vec<Self> {
        TABLE4
            .iter()
            .filter_map(|(l, _)| Self::table4(*l))
            .collect()
    }

    /// Plasticity parameters from Table 3, which the paper gives as seeding
    /// frequency, seeding radius, shade tolerance, `T_A` and `P_A`.
    pub fn with_plasticity(
        mut self,
        seeding_frequency: f32,
        seeding_radius: f32,
        shade_tolerance: f32,
        temperature_optimum: f32,
        precipitation_optimum: f32,
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
            ..Self::table4('a').unwrap()
        }
        .with_plasticity(1.0, 8.0, 0.15, 8.0, 672.0)
        .named("shrub")
    }

    /// Fig. 19's type (II): a conifer.
    pub fn conifer() -> Self {
        Self {
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
            ..Self::table4('f').unwrap()
        }
        .with_plasticity(2.0, 3.0, 0.45, 12.0, 672.0)
        .named("conifer")
    }

    /// Fig. 19's type (III): a broadleaf deciduous tree.
    pub fn deciduous() -> Self {
        Self {
            leaf_color: [0.28, 0.5, 0.14],
            bark_color: [0.4, 0.34, 0.28],
            leaf_size: 0.45,
            max_height: 35.0,
            ..Self::table4('g').unwrap()
        }
        .with_plasticity(1.0, 3.0, 0.6, 16.0, 672.0)
        .named("deciduous")
    }

    /// A grass or forb: tiny, short-lived, many seeds.
    pub fn grass() -> Self {
        Self {
            leaf_color: [0.45, 0.55, 0.2],
            segment_length_max: 0.15,
            leaf_size: 0.12,
            max_height: 0.8,
            heartwood_rate: 0.0,
            ..Self::table4('d').unwrap()
        }
        .with_plasticity(8.0, 4.0, 0.2, 20.0, 600.0)
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
            ..Self::table4('e').unwrap()
        }
        .with_plasticity(1.0, 6.0, 0.2, 27.0, 600.0)
        .named("baobab")
    }

    /// A palm-like single-trunk tree.
    pub fn palm() -> Self {
        Self {
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
            ..Self::table4('c').unwrap()
        }
        .with_plasticity(3.0, 6.0, 0.3, 26.0, 2500.0)
        .named("palm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table4_has_sixteen_rows() {
        assert_eq!(PlantType::table4_all().len(), 16);
        assert!(PlantType::table4('z').is_none());
        let baobab = PlantType::table4('e').unwrap();
        assert_eq!(baobab.apical_control_mature, 0.5);
        assert_eq!(baobab.determinacy_mature, 0.33);
        assert_eq!(baobab.flowering_age, 58.0);
    }
}
