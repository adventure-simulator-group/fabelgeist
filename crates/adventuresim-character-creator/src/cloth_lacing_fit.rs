//! Ring stations and routing are frozen on real cloth edges before refits.
use crate::bodice_lacing_plan::{BodiceLacingPlan, LacingAttachment};
use crate::cloth_closure_fit::edge_frame;
use crate::cloth_fastener_mesh::{FastenerFrame, FastenerMesh};
use crate::cloth_lacing_mesh::ring_center;
use crate::cloth_opening::{ClosureAnchors, ClosureSide};
use crate::device_fit::DevicePiece;
use anyhow::Result;
use fabelgeist_armor::{ClothLacing, LaceRouting};

#[derive(Clone, Copy)]
struct RingAnchor {
    pair: ClosureAnchors,
    side: ClosureSide,
}
struct LaceSpan {
    start: RingAnchor,
    end: RingAnchor,
}
impl DevicePiece {
    pub(crate) fn attach_lacing(
        &mut self,
        plan: &BodiceLacingPlan,
        torso_sources: std::ops::Range<usize>,
        opening: Option<&fabelgeist_armor::gpu::SkirtDressingOpening>,
    ) -> Result<()> {
        let first = torso_sources.start;
        let support = crate::cloth_lacing_support::LacingSupport::new(self, torso_sources, opening);
        let design = &plan.design;
        let stations = match &plan.attachment {
            LacingAttachment::Metal(opening) => {
                let stations = design
                    .positions
                    .iter()
                    .map(|&at| opening.sample(at, first))
                    .collect::<Result<Vec<_>>>()?;
                for anchors in &stations {
                    self.attach_edge_solid(anchors, |part| {
                        FastenerMesh::lacing_rings(design, edge_frame(part, anchors)?)
                    })?;
                }
                stations
            }
            LacingAttachment::Sewn(pairs) => {
                let mut stations = Vec::new();
                for pair in pairs {
                    let anchors = ClosureAnchors {
                        left: pair.left.center.shifted(first),
                        right: pair.right.center.shifted(first),
                    };
                    self.attach_edge_solid(&anchors, |part| {
                        FastenerMesh::sewn_eyelets(design, pair, first, part)
                    })?;
                    stations.push(anchors);
                }
                stations
            }
        };
        for span in route(design.routing, &stations) {
            let anchors = ClosureAnchors {
                left: span.start.pair.side(span.start.side),
                right: span.end.pair.side(span.end.side),
            };
            self.attach_edge_solid(&anchors, |part| {
                let a = edge_frame(part, &span.start.pair)?;
                let b = edge_frame(part, &span.end.pair)?;
                FastenerMesh::lacing_cord(
                    design,
                    FastenerFrame {
                        left: lace_center(
                            design,
                            edge_frame(part, &span.start.pair)?,
                            span.start.side,
                        )?,
                        right: lace_center(
                            design,
                            edge_frame(part, &span.end.pair)?,
                            span.end.side,
                        )?,
                        normal: (a.normal + b.normal).normalize(),
                    },
                    part,
                    &support,
                )
            })?;
        }
        Ok(())
    }
}
fn route(routing: LaceRouting, stations: &[ClosureAnchors]) -> Vec<LaceSpan> {
    let mut spans = Vec::new();
    if routing == LaceRouting::Spiral {
        spans.extend(stations.iter().map(|&pair| LaceSpan {
            start: RingAnchor {
                pair,
                side: ClosureSide::Left,
            },
            end: RingAnchor {
                pair,
                side: ClosureSide::Right,
            },
        }));
    }
    for pair in stations.windows(2) {
        spans.push(LaceSpan {
            start: RingAnchor {
                pair: pair[0],
                side: ClosureSide::Right,
            },
            end: RingAnchor {
                pair: pair[1],
                side: ClosureSide::Left,
            },
        });
        if routing == LaceRouting::Crossed {
            spans.push(LaceSpan {
                start: RingAnchor {
                    pair: pair[0],
                    side: ClosureSide::Left,
                },
                end: RingAnchor {
                    pair: pair[1],
                    side: ClosureSide::Right,
                },
            });
        }
    }
    if routing == LaceRouting::Crossed
        && let Some(&pair) = stations.last()
    {
        // Join both alternating diagonal families at the upper turnaround;
        // the two lower endpoints remain the ends of one cord route.
        spans.push(LaceSpan {
            start: RingAnchor {
                pair,
                side: ClosureSide::Left,
            },
            end: RingAnchor {
                pair,
                side: ClosureSide::Right,
            },
        });
    }
    spans
}

fn lace_center(
    design: &ClothLacing,
    frame: FastenerFrame,
    side: ClosureSide,
) -> Result<fabelgeist_math::Vec3> {
    match design.eyelet {
        fabelgeist_armor::LacingEyelet::Metal { .. } => ring_center(design, frame, side),
        fabelgeist_armor::LacingEyelet::Sewn { .. } => Ok(match side {
            ClosureSide::Left => frame.left,
            ClosureSide::Right => frame.right,
        }),
    }
}
