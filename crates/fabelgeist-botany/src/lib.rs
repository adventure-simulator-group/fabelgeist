//! Plant ecosystems at three scales, after Makowski et al., *Synthetic
//! Silviculture: Multi-scale Modeling of Plant Ecosystems* (SIGGRAPH 2019).
//!
//! * **Module scale** ([`prototype`]): a branch module is an instance of one
//!   of a few self-similar prototypes -- small skeletal graphs after Hallé's
//!   architectural models -- developed deterministically by interpolating its
//!   segments' lengths and diameters with physiological age, and bent by
//!   tropism.
//! * **Plant scale** ([`plant`]): a plant is a tree of modules that
//!   self-organises. Light exposure is gathered basipetally and redistributed
//!   as vigor acropetally under apical control (the extended Borchert-Honda
//!   model, at the scale of modules); vigor sets how fast each module
//!   develops, where new modules attach and which are shed.
//! * **Ecosystem scale** ([`ecosystem`]): plants on a terrain, in a climate,
//!   compete for space through the overlap of their modules' bounding
//!   spheres, shade each other through a propagated shadow grid, orient new
//!   modules away from their neighbours, flower, seed and senesce.
//!
//! Growth is continuous. [`Ecosystem::advance`] moves the simulation on by
//! any span, a frame's worth or years: the discrete decisions of a step --
//! light, vigor, buds breaking, branches shed, orientation, death and seeding
//! -- run once every [`EcosystemSettings::dt`] years ([`Plant::regulate`]),
//! and in between every module lengthens and every stem thickens at the rates
//! the last step set ([`Plant::develop`]). [`Ecosystem::step`] is a whole
//! step at once, and advancing frame by frame arrives at the same forest.
//!
//! The coupling passes -- light and module orientation over every module in
//! the scene -- run on the GPU ([`environment::gpu`]) or on the host with
//! rayon ([`environment::host`]). The per-plant tree logic is irregular and
//! runs on the host, one plant per task. Branch geometry is swept on the GPU
//! by `fabelgeist-tube` straight into mesh buffers.
//!
//! # Where this departs from the paper
//!
//! The paper leaves several constants unstated and some formulas, taken
//! literally, do not produce forests. Every departure is local and named:
//!
//! * **Effective light.** The paper's `Q_eff = lerp(s_tol, 1, Q Q_G)` floors
//!   *all* light at `s_tol`, so a crowded module never starves and crowns
//!   fill in without limit. Here shade tolerance applies to the shadow only:
//!   `Q_eff = Q lerp(s_tol, 1, Q_G)` (see [`ecosystem`]).
//! * **Vigor scale.** The root receives `alpha Q_total` capped at
//!   `v_root_max`, the `alpha` of the Borchert-Honda model the paper extends
//!   ([`PlantType::vigor_scale`]). Without it, laterals under the paper's
//!   strong apical control (`lambda` up to 1) are shed the step after they
//!   appear.
//! * **Crowding.** Eq. 1's overlap is taken relative to the module's own
//!   sphere and skips its parent, children and siblings, whose overlap is
//!   the prototype's own layout.
//! * **Tropism target.** Laterals aim at the species' set-point angle; the
//!   trunk's continuation aims straight up.
//! * **Table 4** is read with `alpha` as a cosine, the signed column as
//!   `g2`, and `phi` in centimetres ([`species`] explains why).
//! * **Flowering** delay `v_root_max / v_root` is capped at two, so that
//!   suppressed plants still flower before they senesce.
//! * **Shadow propagation** is Palubicki's pyramid written as a layer-by-layer
//!   normalised blur ([`environment::shadow`]), so it runs as one dispatch.
//! * **Seeds** do not take root within [`EcosystemSettings::seed_spacing`] of
//!   a standing plant.
//!
//! Mechanisms the paper does not have, added because without them
//! plants grow where no tree could keep them:
//!
//! * **Self-pruning** (branch autonomy): a mature branch whose subtree
//!   averages less actual light than the species' compensation point is
//!   shed, whatever vigor the trunk would send it
//!   ([`PlantType::light_compensation`]). In the paper only vigor below
//!   `v_min` sheds, so a shaded branch lives on the rest of the tree.
//! * **Mechanics**: each module carries the wood and leaves above it; the
//!   bending moment at its base sags it (beam deflection, each module by its
//!   own length's share) and breaks it off past the wood's modulus of
//!   rupture ([`PlantType::wood_strength`]). A shoot grows in the direction
//!   it was oriented, so only sag that comes after it carries it down, and
//!   the trunk stays upright. A limb whose load nears its safe stress
//!   ([`PlantType::safety_factor`]) stops lengthening and branching.
//! * **Hydraulic limit**: the further a shoot is from the roots -- its
//!   height, and the branch between it and the trunk, where water moves
//!   harder ([`PlantType::branch_resistance`]) -- the less turgor it has to
//!   expand with, so growth slows near the plant's ceiling and stops at it
//!   ([`Plant::max_height`]). It bounds both how tall a plant grows and how
//!   far its limbs reach; without it the leader climbs at a steady rate for
//!   as long as the plant lives, and limbs stiffened by heartwood reach out
//!   as long.
//! * **Apical control of elongation**: branches lengthen slower than the
//!   trunk ([`PlantType::lateral_extension`]), and slower still as they age
//!   ([`PlantType::branch_half_life`]). `lambda` only shares out vigor, and
//!   a shoot with more than `v_max` grows no faster, so on its own it cannot
//!   keep an excurrent crown narrow; and the light model barely darkens the
//!   inside of a crown, so without the decline a conifer's lowest limbs grow
//!   on for as long as it lives.
//! * **Clear bole**: a species may keep its trunk bare below a share of its
//!   height ([`PlantType::clear_bole`]), shedding the branches its rising
//!   crown leaves behind -- a baobab's crown gathered atop its trunk.
//! * **Sprouting**: a shrub sends up new stems from its root crown
//!   ([`PlantType::sprouts_per_year`]) up to a clump's worth, as its
//!   reserves allow, so the clump outlives each of its stems.
//! * **Growth that varies**: a shoot grows slower in shade (photosynthesis
//!   saturating with light), and each plant draws its own growth potential
//!   and stature ([`Plant::potential`], [`Plant::stature`]), and grows
//!   shorter where it is poorly adapted. In the paper vigor far above
//!   `v_max` reaches every leader, so every tree of a species grows at the
//!   same rate to the same height.
//! * **Heartwood**: the pipe model gives the sapwood, the wood still
//!   conducting; around a core that only grows -- a share of the sapwood
//!   retired each year ([`PlantType::heartwood_rate`]), and the sapwood shed
//!   branches drew on. Stems thicken ring by ring for as long as the plant
//!   lives, and never thin when it sheds a branch.
//! * **Buildings** ([`obstacles`]): solids are occupied space -- in a
//!   module's crowding, its orientation and the shadow grid -- so crowns
//!   grow round towers and walls, and nothing roots under them.
//! * **Climbers** ([`vine`]): vines that sense, hold on to and climb walls
//!   and trunks, and hang under their own weight where they find nothing.
//! * **Crown shyness**: a bud whose best orientation still leaves it mostly
//!   inside other modules stays dormant
//!   ([`EcosystemSettings::bud_crowding_limit`]). The paper's collision term
//!   only steers a new module, so laterals grew on into neighbouring crowns
//!   the top-down shadow grid barely shades.
//!
//! ```no_run
//! # async fn demo() -> anyhow::Result<()> {
//! use fabelgeist_botany::{Climate, Ecosystem, PlantType, Terrain};
//! let mut ecosystem = Ecosystem::new(
//!     Terrain::flat(60.0, 60.0),
//!     Climate::new(fabelgeist_botany::units::Celsius::from_degrees(12.0), fabelgeist_botany::units::Millimetres::from_millimetres(900.0)),
//!     vec![PlantType::conifer(), PlantType::deciduous(), PlantType::shrub()],
//! );
//! ecosystem.scatter(40);
//! for _ in 0..200 {
//!     ecosystem.step().await?;
//! }
//! let branches = ecosystem.branch_meshes(6);
//! # Ok(()) }
//! ```

pub mod climate;
pub mod ecosystem;
pub mod environment;
pub mod error;
pub mod identity;
pub mod mesh;
pub mod obstacles;
pub mod plant;
pub mod policy;
pub mod prototype;
pub mod rng;
pub mod scenes;
pub mod species;
pub mod terrain;
pub mod units;
pub mod vine;

#[cfg(feature = "gpu")]
pub mod render;

#[cfg(feature = "viewer")]
pub mod viewer;

pub use climate::{Biome, Climate};
pub use ecosystem::{Census, Ecosystem, EcosystemSettings, StepReport};
pub use environment::{EnvironmentSettings, ShadowGrid};
pub use obstacles::Obstacles;
pub use plant::{Module, Node, Plant};
pub use prototype::{Prototype, PrototypeLibrary};
pub use species::PlantType;
pub use terrain::Terrain;
pub use vine::{Climbing, Vine, VineType};
