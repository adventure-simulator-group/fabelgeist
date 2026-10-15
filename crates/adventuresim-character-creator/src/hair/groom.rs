//! A hairstyle grown on the bust, and the same hair placed on a character.

use fabelgeist_hair::collider::{Collider, push_out};
use fabelgeist_hair::lod::Form;
use fabelgeist_hair::ops::{Context, Shrinkwrap};
use fabelgeist_hair::{HairLods, Head, LodSettings, MeshCollider, Strands, Surface};
use fabelgeist_math::Vec3;
use rayon::prelude::*;

use super::{Groom, HeadPlacement, ScalpCap, drape};

/// Edge of the surface-nets cells the scalp is cut from, metres: the size the
/// hair crate's viewer and dataset grow on.
const SCALP_CELL: f32 = 0.004;
/// How far the body collider answers, metres: hair farther from the skin
/// than this is outside and needs no exact distance.
const BODY_REACH: f32 = 0.1;
/// How far off the character's skin a strand's root sits, metres: as far as
/// the hair crate keeps a card's root off its bust.
const ROOT_CLEARANCE: f32 = 0.0005;
/// Steps taken to set a root on the skin along its normal, how close it
/// must come, metres, and how far it may move, metres, before it is pushed
/// straight out instead.
const ROOT_STEPS: usize = 12;
const ROOT_TOLERANCE: f32 = 0.0001;
const MAX_ROOT_SHIFT: f32 = 0.03;
/// The flattest a root's normal may meet the skin and still be followed:
/// the cosine of the angle between them.
const MIN_ROOT_INCIDENCE: f32 = 0.3;

/// A hairstyle grown on the mannequin bust, in the bust's frame: metres, up
/// along +Y, the face towards +Z.
pub struct GroomedHair {
    pub groom: Groom,
    strands: Strands,
    head: Head,
    scalp: Surface,
}

/// The skin hair must stay out of, in model space.
pub struct BodySurface<'a> {
    pub positions: &'a [[f32; 3]],
    pub faces: &'a [[u32; 3]],
}

/// Hair on a character's head, ready to draw at every level of detail.
pub struct PlacedHair {
    pub groom: Groom,
    pub(super) lods: HairLods,
    /// The skin the hair was placed on, which the cards' edges stay out of.
    pub(super) body: MeshCollider,
    placement: HeadPlacement,
    cap: ScalpCap,
}

/// How much hair covers a point of skin, each share 0 to 1: how densely
/// roots grow around it, and how much hair lies over it.
#[derive(Clone, Copy, Debug)]
pub struct SkinCover {
    pub roots: f32,
    pub over: f32,
}

impl GroomedHair {
    /// Grow `groom` on the mannequin bust: about half a second to a second
    /// of every core.
    pub fn grow(groom: Groom) -> Self {
        let look = groom.look();
        let head = look.head(&Head::mannequin());
        let scalp = head.scalp(SCALP_CELL);
        let strands = look.grow(&head, &scalp);
        Self {
            groom,
            strands,
            head,
            scalp,
        }
    }

    /// The bust this hair grew on.
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// Carry the strands onto a character and prepare every level of detail
    /// there, outside `body`. The bust is only stretched to the character's
    /// head; wherever the character's skull is fuller, strands would lie
    /// under its skin. Each strand is first lifted whole until its root sits
    /// on the skin, then shrinkwrapped around the body as the groom is
    /// around the bust, keeping its length; the levels of detail keep their
    /// coarse strands and cards clear of the body too.
    pub fn place(&self, placement: &HeadPlacement, body: &BodySurface<'_>) -> PlacedHair {
        let mut strands = self.strands.clone();
        for point in &mut strands.positions {
            *point = Vec3::from_array(placement.point(point.to_array()));
        }
        for normal in &mut strands.root_normal {
            *normal = Vec3::from_array(placement.normal(normal.to_array()));
        }
        let collider = MeshCollider::new(
            body.positions
                .iter()
                .copied()
                .map(Vec3::from_array)
                .collect(),
            body.faces.to_vec(),
            BODY_REACH,
        );
        lift_roots(&mut strands, &collider);
        drape::drape(
            &mut strands,
            &self.strands,
            &self.head,
            &collider,
            placement,
        );
        let context = Context {
            scalp: &self.scalp,
            collider: &collider,
        };
        Shrinkwrap {
            preserve_length: true,
            ..Shrinkwrap::default()
        }
        .apply(&mut strands, &context);
        self.prepare(&strands, placement, collider)
    }

    /// This hair left on its own bust, as the hair crate grew it, with every
    /// level of detail prepared against the bust: what drawing it on a
    /// character is measured against. The bust's surface comes beside it.
    pub fn on_bust(&self) -> (PlacedHair, Surface) {
        let bust = self.head.mesh(SCALP_CELL);
        let collider =
            MeshCollider::new(bust.positions.clone(), bust.triangles.clone(), BODY_REACH);
        let placement = HeadPlacement::identity(0);
        (self.prepare(&self.strands, &placement, collider), bust)
    }

    /// Every level of detail of `strands`, placed by `placement` against
    /// `body`, and the scalp cap under them.
    fn prepare(
        &self,
        strands: &Strands,
        placement: &HeadPlacement,
        body: MeshCollider,
    ) -> PlacedHair {
        let center = Vec3::from_array(placement.point(self.head.cranium.center.to_array()));
        let lods = HairLods::build(strands, center, Some(&body), &LodSettings::default());
        let cap = ScalpCap::fit(&self.scalp, &self.head, placement, &body, &lods.cover);
        PlacedHair {
            groom: self.groom,
            lods,
            body,
            placement: *placement,
            cap,
        }
    }
}

impl PlacedHair {
    pub fn placement(&self) -> &HeadPlacement {
        &self.placement
    }

    /// The scalp under the hair, fitted to the character's skull.
    pub fn cap(&self) -> &ScalpCap {
        &self.cap
    }

    /// Every form this hair can be drawn in, finest first: each strand
    /// level with all its strands, then each cut of cards.
    pub fn forms(&self) -> Vec<Form> {
        let strands = (0..self.lods.levels.len()).map(|level| Form::Strands { level, share: 1.0 });
        let cards = (0..self.lods.cards.len()).map(|level| Form::Cards { level });
        strands.chain(cards).collect()
    }

    /// Points along each strand at strand `level`.
    pub fn strand_points(&self, level: usize) -> usize {
        self.lods.levels.get(level).map_or(0, |level| level.points)
    }

    /// Strands in the groom.
    pub fn strand_count(&self) -> usize {
        self.lods.strand_count()
    }

    /// The cards in cut `level`.
    pub fn card_count(&self, level: usize) -> usize {
        self.lods.cards.get(level).map_or(0, |cut| cut.cards)
    }
}

/// Move each strand whole along its root's normal until the root sits just
/// outside `body`. The normal is the bust scalp's, carried, and so smooth:
/// pushing each root to the nearest point of the body instead would move
/// neighbouring roots apart along the normals of different facets wherever
/// the body is coarse, and part the hair along its edges -- at the nape,
/// where roots start up to a centimetre and a half inside, into wedges.
fn lift_roots(strands: &mut Strands, body: &MeshCollider) {
    let points = strands.points;
    let normals = &strands.root_normal;
    strands
        .positions
        .par_chunks_mut(points)
        .zip(normals.par_iter())
        .for_each(|(strand, &normal)| {
            let lift = root_lift(body, strand[0], normal);
            for point in strand {
                *point += lift;
            }
        });
}

/// How far to move `root` along `normal` to sit [`ROOT_CLEARANCE`] outside
/// `body`: out of it, or down onto it. Where the normal runs too nearly
/// along the skin to reach it, the root is pushed straight out instead.
fn root_lift(body: &MeshCollider, root: Vec3, normal: Vec3) -> Vec3 {
    let mut along = 0.0;
    for _ in 0..ROOT_STEPS {
        let (distance, out) = body.query(root + normal * along);
        let error = ROOT_CLEARANCE - distance;
        if error.abs() < ROOT_TOLERANCE {
            return normal * along;
        }
        // The distance changes along the normal as fast as the normal
        // meets the skin's.
        along += error / normal.dot(out).max(MIN_ROOT_INCIDENCE);
        if along.abs() > MAX_ROOT_SHIFT {
            break;
        }
    }
    push_out(body, root, ROOT_CLEARANCE) - root
}
