//! Space a needlework chart by the cloth it covers.
//!
//! A band fitted round the body is charted by angle about an axis. Where the
//! body is not round about that axis, equal angles span unequal lengths of
//! cloth, and ornament drawn on the chart is stretched on one side and
//! squeezed on the other: round pearls on a collar become ovals at the
//! throat. Respacing the chart's `across` coordinate by the cloth it spans
//! keeps the ornament's proportions all the way round.
use fabelgeist_armor::GeneratedArmor;

/// Intervals of the `across` coordinate over which cloth length is averaged.
/// They are finer than any body feature a band passes over and coarser than
/// the triangles of the bands that carry needlework.
const INTERVALS: usize = 64;

/// Respace `across` on every component that carries needlework, so equal
/// steps of the chart span equal lengths of cloth. Where the cloth's length
/// per step also changes along the band, as on a cone, the average is used.
/// Only the cloth that carries ornament is measured: a bodice is not as wide
/// at its armholes as at the girdle that its ornament lies on.
pub(crate) fn even_out(armor: &mut GeneratedArmor) {
    for component in &armor.components {
        let Some(needlework) = component
            .textile
            .as_ref()
            .and_then(|fabric| fabric.needlework.as_ref())
        else {
            continue;
        };
        let ornamented = needlework
            .strokes
            .iter()
            .map(|stroke| stroke.region.along_interval())
            .collect::<Vec<_>>();
        let across = |vertex: usize| armor.texcoords[vertex][0];
        let (first, last) = component
            .vertices
            .clone()
            .map(across)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), value| {
                (low.min(value), high.max(value))
            });
        let extent = last - first;
        if extent.partial_cmp(&0.) != Some(std::cmp::Ordering::Greater) {
            continue;
        }
        let interval_of = |value: f32| (value - first) / extent * INTERVALS as f32;
        // Cloth length per unit of `across`, averaged over each interval and
        // weighted by how much of the interval each triangle spans.
        let mut length = [0_f32; INTERVALS];
        let mut weight = [0_f32; INTERVALS];
        let (triangles, _) = armor.indices[component.indices.clone()].as_chunks::<3>();
        for triangle in triangles {
            let corners = triangle.map(|vertex| vertex as usize);
            let along = corners
                .iter()
                .map(|&vertex| armor.texcoords[vertex][1])
                .sum::<f32>()
                / 3.;
            if !ornamented
                .iter()
                .any(|[start, end]| (*start..=*end).contains(&along))
            {
                continue;
            }
            let Some(stretch) = stretch_across(armor, corners) else {
                continue;
            };
            let spans = corners.map(|vertex| interval_of(across(vertex)));
            let low = spans.into_iter().fold(f32::INFINITY, f32::min);
            let high = spans.into_iter().fold(f32::NEG_INFINITY, f32::max);
            for (interval, (length, weight)) in length.iter_mut().zip(&mut weight).enumerate() {
                let start = interval as f32;
                let overlap = (high.min(start + 1.) - low.max(start)).max(0.);
                *length += stretch * overlap;
                *weight += overlap;
            }
        }
        let covered = weight.iter().filter(|&&weight| weight > 0.).count();
        if covered == 0 {
            continue;
        }
        let mean = length.iter().sum::<f32>() / weight.iter().sum::<f32>();
        // Length of cloth from the chart's first edge to the end of each
        // interval. An interval no triangle spans takes the mean.
        let mut travelled = [0_f32; INTERVALS];
        let mut total = 0.;
        for interval in 0..INTERVALS {
            total += if weight[interval] > 0. {
                length[interval] / weight[interval]
            } else {
                mean
            };
            travelled[interval] = total;
        }
        if total <= 0. {
            continue;
        }
        let texcoords = &mut armor.texcoords[component.vertices.clone()];
        for texcoord in texcoords {
            let position = interval_of(texcoord[0]).clamp(0., INTERVALS as f32);
            let interval = (position as usize).min(INTERVALS - 1);
            let before = if interval == 0 {
                0.
            } else {
                travelled[interval - 1]
            };
            let within = position - interval as f32;
            let cloth = before + (travelled[interval] - before) * within;
            texcoord[0] = first + extent * cloth / total;
        }
    }
}

/// Metres of cloth per unit of `across` on one triangle, or `None` when the
/// triangle is degenerate in the chart.
fn stretch_across(armor: &GeneratedArmor, corners: [usize; 3]) -> Option<f32> {
    let [a, b, c] = corners.map(|vertex| armor.positions[vertex]);
    let [ta, tb, tc] = corners.map(|vertex| armor.texcoords[vertex]);
    let (du1, dv1) = (tb[0] - ta[0], tb[1] - ta[1]);
    let (du2, dv2) = (tc[0] - ta[0], tc[1] - ta[1]);
    let determinant = du1 * dv2 - dv1 * du2;
    if determinant == 0. {
        return None;
    }
    let mut squared = 0.;
    for axis in 0..3 {
        let (e1, e2) = (b[axis] - a[axis], c[axis] - a[axis]);
        squared += ((e1 * dv2 - e2 * dv1) / determinant).powi(2);
    }
    let stretch = squared.sqrt();
    stretch.is_finite().then_some(stretch)
}
