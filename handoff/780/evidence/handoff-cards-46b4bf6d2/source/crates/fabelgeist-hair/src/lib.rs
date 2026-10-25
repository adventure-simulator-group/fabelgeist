//! Strand hair after Rosu et al., *DiffLocks: Generating 3D Hair from a
//! Single Image using Diffusion Models* (CVPR 2025) -- the synthetic-data
//! half of it: how its 40K hairstyles are made, and then how to draw them,
//! from film to phone.
//!
//! # Making hair
//!
//! DiffLocks grows every hairstyle in Blender with geometry nodes. A base
//! style is a few dozen hand-drawn guide strands; a stack of Blender's hair
//! node groups interpolates them to ~100K strands over the scalp, then
//! clumps, curls and noises them -- each several times at different scales
//! -- shrinkwraps them off the body and drapes them under random gravity and
//! wind. 110 parameters, each drawn from a range, make every sample
//! different. The same pipeline here:
//!
//! * [`Strands`]: curves of equal point count with the attributes the node
//!   groups pass between them (root UV and normal, a stable ID, the parting
//!   group, the guide map).
//! * [`ops`]: the node groups themselves, written out from their node
//!   graphs -- `Interpolate Hair Curves`, `Clump Hair Curves`, `Curl Hair
//!   Curves`, `Frizz Hair Curves`, `Hair Curves Noise`, `Trim`, `Smooth`,
//!   `Shrinkwrap Hair Curves` with `Restore Curve Segment Length`, `Create
//!   Guide Index Map`, `Set Hair Curve Profile` and their `shape_range`
//!   falloff -- plus waves ([`ops::Wave`]), each strand's root turned to its
//!   follicle ([`ops::Orient`]) and the drape ([`ops::Settle`]), run in
//!   parallel over strands.
//! * [`Groom`]: a stack of operations, serialisable, the recipe of one
//!   hairstyle; [`Look`] a stack built from the few knobs that mean
//!   something to a person, in DiffLocks' order, and drawn at random the way
//!   the dataset draws.
//! * [`Hairdo`]: what shapes the guides, as three choices made apart --
//!   the [`Follicles`] (the growth pattern, [`Growth`]: a crown whorl, two,
//!   a cowlick, forward, back...; the angle hair leaves the scalp at, how
//!   far it keeps its follicle's direction, how thick it is, how evenly it
//!   covers the scalp, the [`Hairline`]'s shape), the [`Haircut`] (lengths
//!   over the head, a hemline, clippers, a fringe, a dome: [`Cut`] names
//!   the usual ones) and the [`Hairstyle`] (parted, swept, slicked back,
//!   stood up, tied: [`Arrangement`]) -- grown as guides on a [`Head`] (a
//!   mannequin bust as a distance field, which is also the scalp,
//!   [`Surface`], and the collider, [`Collider`]). [`Preset`] names
//!   twenty-four hairdos: every kind of hair the paper's figures show
//!   ([`Preset::PAPER`]: bob, crop, long, shag, bowl cut, ringlets, afro,
//!   quiff, balding and more) and some it lacks (ponytail, bun, braid,
//!   dreadlocks, mohawk).
//! * [`HairMaterial`]: the Chiang hair BSDF's parameters, melanin and
//!   redness as Blender's Principled Hair takes them.
//! * [`dataset`] and the `hair-dataset` binary: samples on disk in the
//!   dataset's own layout (`full_strands.npz`, the density map, metadata,
//!   a render), and [`export`]: NumPy, USC-HairSalon `.data`, PLY scalps.
//!
//! # Drawing hair
//!
//! [`HairLods`] prepares a groom once into every form it can be drawn in --
//! the strands at several point counts in one shuffled order (so any prefix
//! is an even sample, and fewer strands is just a smaller count), hair cards
//! cut from the groom's own locks with a painted atlas, and occlusion baked
//! into every point. A [`LodPolicy`] picks the form from the head's size on
//! screen for a target -- cinematic, desktop or mobile -- with its shading.
//! [`render::HairRenderer`] draws it: camera-facing ribbons from a storage
//! buffer, Marschner's lobes in Karis' fit (or Kajiya-Kay on mobile), deep
//! opacity maps from the key light, stochastic transparency so sub-pixel
//! strands add up to their true density, and a jittered running mean that
//! converges to a supersampled still. `hair-viewer` shows it all live.
//!
//! # Moving hair
//!
//! [`sim::HairSim`] runs real-time physics, optionally: some of the groom's
//! strands simulated as elastic rods ([`rod`]: Cosserat rods with hair's
//! own stiffness and mass, under gravity, in the air, clamped in follicles
//! a moving head carries, kept off its distance field, each step implicit
//! and solved exactly; the groom their equilibrium under gravity), every
//! drawn strand following the ones nearest its root in a compute pass, and
//! moving frames blended over time so the stochastic coverage does not
//! sparkle. On the GPU
//! ([`gpu::GpuSim`]) the strands also meet: hair denser than at rest is
//! pushed apart (its volume kept), hair moving through hair drags it along,
//! and strands collide, segment against segment. How many strands, of how
//! many points, is a level of detail of its own ([`sim::SimLevel`]), chosen
//! by the head's size on screen like the drawing's.
//!
//! # On the GPU
//!
//! All of it runs on the GPU as well ([`gpu`], the `gpu` feature): the
//! guides, the groom, its levels of detail and cards, the head's cover and
//! the physics are compute kernels on the compute layer, and what they
//! make is drawn from the buffers they wrote -- a hundred thousand strands
//! grown and prepared in tens of milliseconds, where the host takes a
//! second or two; the scalp they root on is cut there too, when the
//! hairline changes. The viewer, the dataset and the snapshots groom
//! there. The host path stays, the reference the kernels are tested
//! against (the scalp to its triangles, the guides to a micrometre) and the
//! way without a GPU.
//!
//! # Where this departs from the paper
//!
//! * **The body.** DiffLocks grows hair on the SMPL-X scalp and collides it
//!   with the SMPL-X body; SMPL-X is not ours to ship. The [`Head`] is a
//!   bust of unioned ellipsoids and capsules; any scalp with a UV map loads
//!   in its place ([`Surface::from_ply`] reads DiffLocks' `scalp.ply`).
//! * **The base styles** are 75 hand-drawn sets of guides in the paper and
//!   not published. Here each style is grown: every guide leaves its
//!   follicle as the growth pattern points, rises to the layer of hair
//!   beneath it, lies along the head as combed while it is supported, and
//!   bends as a cantilever under its own weight once free, until the cut
//!   ends it ([`hairdo`]). Every strand's root is then turned to its own
//!   follicle ([`ops::Orient`]), which interpolating guides a few
//!   centimetres apart would blur.
//! * **Interpolation** takes a strand count rather than a density, fades
//!   `Follow Surface Normal`'s turn out along the strand (Blender turns the
//!   whole strand, which on a curved scalp swings long hair's tips out), and
//!   blends only guides whose courses agree: the mean of a guide combed
//!   left and one combed right would run down the face between them.
//! * **The drape** is each strand's static equilibrium as an elastic rod
//!   against the head's field ([`ops::Settle`]), not Blender's cloth solver,
//!   and applies the *change* from standard gravity and still air -- so it
//!   blows and lifts a style without sagging it into another. It runs on
//!   the guides, and the full groom inherits it.
//! * **Frizz and noise** are Blender's, with two changes: frizz takes a
//!   share of flyaway strands (the rest barely wander) and can wander
//!   smoothly, and the look uses noise as offsets, not Blender's cumulative
//!   walk, whose reach grows with the point count.
//! * **Rendering.** DiffLocks path-traces the Chiang BSDF under HDRIs.
//!   Here rendering is rasterised and real time; the Chiang absorption is
//!   exported with every sample for an offline renderer, and the real-time
//!   colour is Chiang's colour mapping over a quarter of it, matched to
//!   Cycles by eye ([`HairMaterial::base_color`]).
//!
//! ```no_run
//! use fabelgeist_hair::{Arrangement, Cut, Growth, HairLods, Haircut, Head, LodSettings, Look, Preset};
//!
//! // A long cut worn in a ponytail, from a double crown.
//! let mut look = Look::new(Preset::Long);
//! look.hairdo.follicles.growth = Growth::DoubleWhorl;
//! look.hairdo.haircut = Haircut::new(Cut::Layered);
//! look.hairdo.hairstyle = fabelgeist_hair::Hairstyle::new(Arrangement::Ponytail);
//!
//! let head = look.head(&Head::mannequin());
//! let scalp = head.scalp(0.004);
//! let hair = look.grow(&head, &scalp);
//! let lods = HairLods::build(&hair, head.cranium.center, Some(&head), &LodSettings::default())?;
//! # Ok::<(), fabelgeist_hair::lod::LodError>(())
//! ```

pub mod cards;
pub mod collider;
pub mod cover;
pub mod dataset;
pub mod difflocks;
pub mod export;
pub mod follicles;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod grid;
pub mod groom;
pub mod haircut;
pub mod hairdo;
pub mod hairstyle;
pub mod head;
pub mod identity;
pub mod lod;
pub mod look;
pub mod material;
pub mod noise;
pub mod occlusion;
pub mod ops;
pub mod parameters;
pub mod presets;
#[cfg(feature = "render")]
pub mod render;
pub mod rng;
pub mod rod;
pub mod sim;
pub mod strands;
pub mod surface;

pub use collider::{Collider, MeshCollider};
pub use follicles::{Follicles, Growth, Hairline};
pub use groom::Groom;
pub use haircut::{Cut, Fringe, Haircut};
pub use hairdo::Hairdo;
pub use hairstyle::{Arrangement, Hairstyle};
pub use head::Head;
pub use lod::{Blend, HairLods, LodCounts, LodError, LodPolicy, LodResult, LodSettings, Representation};
pub use look::Look;
pub use material::{HairMaterial, Taper};
pub use presets::Preset;
pub use strands::Strands;
pub use surface::Surface;
