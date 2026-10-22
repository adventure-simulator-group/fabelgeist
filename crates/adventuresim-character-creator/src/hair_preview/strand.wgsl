// How hair is lit, wherever it is drawn -- strand ribbons, cards, and the
// strands painted on the scalp: a strand faces the eye whichever way the
// ribbon, card or scalp it is drawn on does.

#define_import_path adventuresim::strand

// How far a strand's shading normal leans from facing the eye towards the
// face it is drawn on, as the hair crate shades cards: enough to round a
// lock.
const FACE_LEAN: f32 = 0.35;
// How far off the line of sight a strand must turn, as the sine of the
// angle, before it is shaded as seen from the side. Seen end-on -- an afro's
// strands at its crown, pointing at the eye -- the direction square to the
// strand facing the eye is no direction at all: it jumps from pixel to pixel
// and lies across the line of sight, where the environment's reflection is
// brightest, and the strands glint a cold grey. A card is seen edge-on then
// too, so its face is no better; nearer end-on the normal turns to the eye
// instead, as the strands' tips face it.
const SIDE_VIEW: f32 = 0.5;

// The shading normal of a strand running along `tangent`, seen from the
// direction `eye`, drawn on a face whose normal is `face` (all unit, the
// face's towards the eye): facing the eye square to the strand, turned a
// little towards the face, and to the eye as the strand turns to point at
// it.
fn strand_normal(tangent: vec3<f32>, eye: vec3<f32>, face: vec3<f32>) -> vec3<f32> {
    let facing = eye - tangent * dot(eye, tangent);
    let side = length(facing);
    if (side < 1e-4) {
        return eye;
    }
    let seen = smoothstep(0.0, SIDE_VIEW, side);
    let square = mix(eye, facing / side, seen);
    return normalize(square + face * FACE_LEAN * seen);
}
