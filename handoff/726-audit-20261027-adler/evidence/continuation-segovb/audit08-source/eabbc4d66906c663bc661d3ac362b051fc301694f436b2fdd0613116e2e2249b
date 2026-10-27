//! Native WGSL hairdo record: metres in the head frame, shader discriminants.
use super::*;
pub(super) struct HairdoUpload {
    pub(super) gpu: HairdoGpu,
    pub(super) axis: Vec<[f32; 4]>,
}
impl HairdoUpload {
    pub(super) fn new(
        hairdo: &Hairdo,
        head: &Head,
        variation: f32,
        seed: u64,
        count: u32,
        points: u32,
        max_path: u32,
    ) -> Self {
        let field = hairdo.follicles.field(head);
        let whorl = |i: usize| field.whorls.get(i).copied();
        let (w0, w1) = (whorl(0), whorl(1));
        let (flow, flow_value) = hairdo.flow_code();
        let tied = Tied::new(head, hairdo);
        let axis: Vec<[f32; 4]> = tied.as_ref().map_or(vec![[0.0; 4]; 2], |t| {
            t.axis.iter().map(|p| [p.x, p.y, p.z, 0.0]).collect()
        });
        let cut = &hairdo.haircut;
        let style = &hairdo.hairstyle;
        let follicles = &hairdo.follicles;
        let fringe = cut.fringe.unwrap_or_default();
        let to3 = |v: Vec3| v.to_array();
        let gpu = HairdoGpu {
            guides: count,
            points,
            max_path,
            key: guide_key(seed),
            variation,
            angle: follicles.angle,
            angle_spread: follicles.angle_spread,
            direction_spread: follicles.direction_spread,
            kink: follicles.kink,
            growth: Growth::ALL
                .iter()
                .position(|g| *g == field.growth)
                .unwrap_or(0) as u32,
            whorls: field.whorls.len() as u32,
            fringe_falls: u32::from(hairdo.fringe_falls()),
            center: to3(field.center),
            lanes: match hairdo.parting() {
                Parting::Lanes { count } => count,
                _ => 0,
            },
            w0: w0.map_or([0.0; 3], |w| to3(w.point)),
            w0_spiral: w0.map_or(0.0, |w| w.spiral),
            w0_hand: w0.map_or(1.0, |w| w.hand),
            w0_reach: w0.map_or(0.07, |w| w.reach),
            w1: w1.map_or([0.0; 3], |w| to3(w.point)),
            w1_spiral: w1.map_or(0.0, |w| w.spiral),
            w1_hand: w1.map_or(1.0, |w| w.hand),
            w1_reach: w1.map_or(0.07, |w| w.reach),
            top: cut.top,
            front: cut.front,
            sides: cut.sides,
            back: cut.back,
            has_hemline: u32::from(cut.hemline.is_some()),
            hemline: cut.hemline.unwrap_or(0.0),
            slope: cut.slope,
            clip: cut.clip,
            clipper: cut.clipper,
            fade: cut.fade,
            strip: cut.strip,
            has_fringe: u32::from(cut.fringe.is_some()),
            fringe_width: fringe.width,
            fringe_depth: fringe.depth,
            fringe_drop: fringe.drop,
            dome: cut.dome,
            flow,
            flow_value,
            part: style.part,
            hold: style.hold,
            lift: style.lift,
            volume: style.volume,
            cling: style.cling,
            tuck: style.tuck,
            hairline: head.hairline,
            recession: head.recession,
            peak: head.peak,
            tied: u32::from(tied.is_some()),
            tie: tied.as_ref().map_or([0.0; 3], |t| to3(t.point)),
            out: tied.as_ref().map_or([0.0; 3], |t| to3(t.out)),
            axis_len: axis.len() as u32,
        };
        Self { gpu, axis }
    }
}
