use super::*;
use crate::city_layout::grounding::tests::Fixture;

#[test]
fn goslar_street_apron_join_is_internal_to_the_owned_union() {
    let fixture = Fixture::load();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let surface = plan.support_surface();
    let plot = fixture.property.plot;
    for segment in exterior(&surface.clipping_outlines) {
        let on_front = segment.iter().all(|point| {
            let point = plot
                .orientation
                .world_to_local(point.as_vec2() - plot.centre_metres);
            (point.y + plot.dimensions_metres.y * 0.5).abs() < 0.001
        });
        if on_front {
            let interval =
                segment_interval(segment, surface.clipping_outlines.last().unwrap(), 0.0);
            assert!(
                interval.is_none_or(|(begin, end)| (end - begin)
                    * (segment[1] - segment[0]).length()
                    < 0.001),
                "internal apron join {segment:?}, interval {interval:?}"
            );
        }
    }
}

#[test]
fn goslar_cut_faces_do_not_cross_the_internal_street_apron_join() {
    let fixture = Fixture::load();
    let source = fixture.source();
    let plan = fixture.selected_plan(&source);
    let foundation = plan
        .foundations(
            &source,
            crate::city_layout::grounding::FoundationEmbedment::from_metres(0.2).unwrap(),
        )
        .unwrap();
    let plot = fixture.property.plot;
    let entry = plan.street_entry.as_ref().unwrap();
    let local = plot
        .orientation
        .world_to_local(entry.reservation.centre_metres - plot.centre_metres);
    for face in &foundation.cut_faces {
        let midpoint = face.iter().copied().sum::<bevy::math::Vec3>() / 3.0;
        let point = plot
            .orientation
            .world_to_local(bevy::math::Vec2::new(midpoint.x, midpoint.z) - plot.centre_metres);
        let face_local: Vec<_> = face
            .iter()
            .map(|p| {
                plot.orientation
                    .world_to_local(bevy::math::Vec2::new(p.x, p.z) - plot.centre_metres)
            })
            .collect();
        let minimum = face_local.iter().copied().fold(
            bevy::math::Vec2::splat(f32::INFINITY),
            bevy::math::Vec2::min,
        );
        let maximum = face_local.iter().copied().fold(
            bevy::math::Vec2::splat(f32::NEG_INFINITY),
            bevy::math::Vec2::max,
        );
        // A side bank meeting the corner of the join is exterior. Only a face
        // running across the frontage can close the internal doorway route.
        if (maximum - minimum).x < (maximum - minimum).y {
            continue;
        }
        assert!(
            (point.y + plot.dimensions_metres.y * 0.5).abs() > 0.001
                || (point.x - local.x).abs() >= entry.reservation.dimensions_metres.x * 0.5,
            "cut face crosses internal apron join {point:?}: {face:?}"
        );
    }
}
