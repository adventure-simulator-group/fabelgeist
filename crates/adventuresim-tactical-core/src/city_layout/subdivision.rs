//! Staggered old-quarter lanes meet shared seams at real T-junctions.
use super::*;

const SITE_HALF_WIDTH_METRES: f32 = CITY_RADIUS_X_METRES;
const EXTENSION_START_METRES: f32 = 300.0;
const EXTENSION_BLOCK_METRES: f32 = 75.0;
const OLD_BLOCK_MIN_METRES: f32 = 70.0;
const OLD_BLOCK_VARIATION_METRES: f32 = 55.0;
const MARKET_LENGTH_METRES: f32 = 150.0;
const JUNCTION_SPACING_METRES: f32 = 2.0;
const STRIP_DEPTH_METRES: f32 = 96.0;
const OLD_SEAM_OFFSET_METRES: f32 = 18.0;
const OLD_SEAM_PHASE: f32 = 1.7;

pub(super) fn build(site: &CitySite, seed: u64, extent: DevelopmentExtent) -> StreetGraph {
    let strip_count = extent.strip_count;
    let market_strip = extent.market_strip();
    let cuts = (0..strip_count)
        .map(|strip| strip_cuts(site, seed, strip, market_strip))
        .collect::<Vec<_>>();
    let seams = (0..=strip_count)
        .map(|seam| {
            let mut points = Vec::new();
            for strip in &cuts[seam.saturating_sub(1)..=seam.min(strip_count - 1)] {
                points.extend_from_slice(strip);
            }
            points.sort_by(f32::total_cmp);
            points.dedup();
            points
        })
        .collect::<Vec<_>>();
    let mut graph = StreetGraph::default();
    for (strip, splits) in cuts.iter().enumerate() {
        for (index, interval) in splits.windows(2).enumerate() {
            let [left, right] = [interval[0], interval[1]];
            let block = CityBlock {
                id: BlockId(((strip as u64) << 32) | index as u64),
                corners: [
                    point(site, strip, left, market_strip),
                    point(site, strip, right, market_strip),
                    point(site, strip + 1, right, market_strip),
                    point(site, strip + 1, left, market_strip),
                ],
                streets: [
                    seam_class(strip, market_strip),
                    StreetClass::Lane,
                    seam_class(strip + 1, market_strip),
                    StreetClass::Lane,
                ],
                market: false,
            };
            for seam in [strip, strip + 1] {
                let boundary = seams[seam]
                    .iter()
                    .copied()
                    .filter(|x| *x >= left && *x <= right)
                    .collect::<Vec<_>>();
                for ends in boundary.windows(2) {
                    graph.segment(
                        [
                            point(site, seam, ends[0], market_strip),
                            point(site, seam, ends[1], market_strip),
                        ],
                        seam_class(seam, market_strip),
                        Some(block.id),
                    );
                }
            }
            for x in [left, right] {
                graph.segment(
                    [
                        point(site, strip, x, market_strip),
                        point(site, strip + 1, x, market_strip),
                    ],
                    StreetClass::Lane,
                    Some(block.id),
                );
            }
            graph.blocks.push(block);
        }
    }
    connect_market_and_approaches(&mut graph, site, market_strip);
    graph
}

fn connect_market_and_approaches(graph: &mut StreetGraph, site: &CitySite, market_strip: usize) {
    let market = graph
        .blocks
        .iter_mut()
        .filter(|block| block.streets[0] == StreetClass::TradeRoute)
        .filter(|block| block.corners[0].distance(block.corners[1]) >= OLD_BLOCK_MIN_METRES)
        .min_by(|a, b| {
            a.centre()
                .length_squared()
                .total_cmp(&b.centre().length_squared())
        })
        .expect("the surveyed spine has a full-width market site");
    market.market = true;
    for side in [-1.0, 1.0] {
        let mut approach = site
            .alignment
            .iter()
            .copied()
            .filter(|anchor| anchor.x * side > SITE_HALF_WIDTH_METRES)
            .collect::<Vec<_>>();
        approach.sort_by(|a, b| (a.x * side).total_cmp(&(b.x * side)));
        approach.insert(
            0,
            point(
                site,
                market_strip,
                side * SITE_HALF_WIDTH_METRES,
                market_strip,
            ),
        );
        for segment in approach.windows(2) {
            graph.segment([segment[0], segment[1]], StreetClass::TradeRoute, None);
        }
    }
}

fn seam_class(seam: usize, market_strip: usize) -> StreetClass {
    if seam == market_strip {
        StreetClass::TradeRoute
    } else {
        StreetClass::Secondary
    }
}

fn point(site: &CitySite, seam: usize, x: f32, market_strip: usize) -> Vec2 {
    let offset = seam as i32 - market_strip as i32;
    let regular = offset as f32 * STRIP_DEPTH_METRES;
    // Unequal old strips retain deep working plots; the eastern extension has
    // shared, regular seams. Transition occurs only between authored breakpoints.
    let old = regular + (offset as f32 * OLD_SEAM_PHASE).sin() * OLD_SEAM_OFFSET_METRES;
    let extension = (x / EXTENSION_START_METRES).clamp(0.0, 1.0);
    Vec2::new(x, old + (regular - old) * extension + site.route_height(x))
}

fn strip_cuts(site: &CitySite, seed: u64, strip: usize, market_strip: usize) -> Vec<f32> {
    let mut anchors = vec![
        -SITE_HALF_WIDTH_METRES,
        0.0,
        MARKET_LENGTH_METRES,
        EXTENSION_START_METRES,
        SITE_HALF_WIDTH_METRES,
    ];
    anchors.extend(
        site.alignment
            .iter()
            .map(|p| p.x)
            .filter(|x| x.abs() < SITE_HALF_WIDTH_METRES),
    );
    anchors.sort_by(f32::total_cmp);
    anchors.dedup();
    let mut cuts = Vec::new();
    for span in anchors.windows(2) {
        let [left, right] = [span[0], span[1]];
        cuts.push(left);
        if strip == market_strip && left == 0.0 && right == MARKET_LENGTH_METRES {
            continue;
        }
        let sample = StreamId::new("city.street-span")
            .seed(seed, &[strip as u64, u64::from(left.to_bits())])
            .to_u64();
        let length = if left >= EXTENSION_START_METRES {
            EXTENSION_BLOCK_METRES
        } else {
            OLD_BLOCK_MIN_METRES
                + StreamId::new("city.block-length")
                    .rng(sample, &[])
                    .inclusive_unit_f32()
                    * OLD_BLOCK_VARIATION_METRES
        };
        let count = ((right - left) / length).round().max(1.0) as usize;
        for index in 1..count {
            let fraction = index as f32 / count as f32;
            let irregular = if left >= EXTENSION_START_METRES {
                0.0
            } else {
                StreamId::new("city.street-jitter")
                    .rng(sample, &[index as u64])
                    .range_f32(-1.0, 1.0)
                    * 0.18
                    / count as f32
            };
            let distance = (right - left) * (fraction + irregular);
            let distance = if left >= EXTENSION_START_METRES {
                distance
            } else {
                (distance / JUNCTION_SPACING_METRES).round() * JUNCTION_SPACING_METRES
            };
            cuts.push(left + distance);
        }
    }
    cuts.push(SITE_HALF_WIDTH_METRES);
    cuts
}
