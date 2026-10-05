//! Binary products retain the exact index without per-node maps or enum strings.
use super::*;
use serde::{Deserializer, Serializer, de::Error};

// Foundation owner indexes cannot exhaust u32 within the product's byte bound.
const NATURAL_OWNER: u32 = u32::MAX;

#[derive(Serialize, Deserialize)]
struct PackedIndex {
    #[serde(with = "crate::geometry_transport::binary")]
    nodes: Vec<[u32; 7]>,
    #[serde(with = "crate::geometry_transport::binary")]
    triangles: Vec<[u32; 2]>,
}

impl Serialize for SupportQueryIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            return self.0.serialize(serializer);
        }
        let index = |value| u32::try_from(value).map_err(serde::ser::Error::custom);
        let mut packed = PackedIndex {
            nodes: Vec::with_capacity(self.0.len()),
            triangles: Vec::new(),
        };
        for node in &self.0 {
            let [kind, first, count] = match &node.children {
                QueryChildren::Branch { left, right } => [0, index(*left)?, index(*right)?],
                QueryChildren::Leaf(references) => {
                    let first = index(packed.triangles.len())?;
                    for reference in references {
                        packed.triangles.push(match *reference {
                            SupportTriangleRef::Natural(triangle) => {
                                [NATURAL_OWNER, index(triangle)?]
                            }
                            SupportTriangleRef::Foundation { owner, triangle } => {
                                [index(owner)?, index(triangle)?]
                            }
                        });
                    }
                    [1, first, index(references.len())?]
                }
            };
            packed.nodes.push([
                node.minimum.x.to_bits(),
                node.minimum.y.to_bits(),
                node.maximum.x.to_bits(),
                node.maximum.y.to_bits(),
                kind,
                first,
                count,
            ]);
        }
        packed.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SupportQueryIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            return Vec::deserialize(deserializer).map(Self);
        }
        let packed = PackedIndex::deserialize(deserializer)?;
        packed.decode().map_err(D::Error::custom)
    }
}

#[derive(Debug, thiserror::Error)]
enum DecodeIssue {
    #[error("invalid support query bounds")]
    Bounds,
    #[error("invalid support query leaf range")]
    LeafRange,
    #[error("invalid support query child tag or branch")]
    Children,
}

impl PackedIndex {
    fn decode(self) -> Result<SupportQueryIndex, DecodeIssue> {
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for (parent, &[x0, y0, x1, y1, kind, first, count]) in self.nodes.iter().enumerate() {
            let minimum = Vec2::new(f32::from_bits(x0), f32::from_bits(y0));
            let maximum = Vec2::new(f32::from_bits(x1), f32::from_bits(y1));
            if !minimum.is_finite() || !maximum.is_finite() || minimum.cmpgt(maximum).any() {
                return Err(DecodeIssue::Bounds);
            }
            let children = match kind {
                0 if first as usize > parent
                    && count as usize > parent
                    && first != count
                    && (first as usize) < self.nodes.len()
                    && (count as usize) < self.nodes.len() =>
                {
                    QueryChildren::Branch {
                        left: first as usize,
                        right: count as usize,
                    }
                }
                1 => {
                    let range = (first as usize)
                        .checked_add(count as usize)
                        .and_then(|end| self.triangles.get(first as usize..end))
                        .ok_or(DecodeIssue::LeafRange)?;
                    QueryChildren::Leaf(
                        range
                            .iter()
                            .map(|&[owner, triangle]| {
                                if owner == NATURAL_OWNER {
                                    SupportTriangleRef::Natural(triangle as usize)
                                } else {
                                    SupportTriangleRef::Foundation {
                                        owner: owner as usize,
                                        triangle: triangle as usize,
                                    }
                                }
                            })
                            .collect(),
                    )
                }
                _ => return Err(DecodeIssue::Children),
            };
            nodes.push(QueryNode {
                minimum,
                maximum,
                children,
            });
        }
        Ok(SupportQueryIndex(nodes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_index_preserves_branching_bounds_and_exact_triangle_references() {
        let leaf = |references| QueryNode {
            minimum: Vec2::new(-0.0, -17.25),
            maximum: Vec2::new(43.75, 10.125),
            children: QueryChildren::Leaf(references),
        };
        let index = SupportQueryIndex(vec![
            QueryNode {
                minimum: Vec2::splat(-20.0),
                maximum: Vec2::splat(50.0),
                children: QueryChildren::Branch { left: 1, right: 2 },
            },
            leaf((0..8).map(SupportTriangleRef::Natural).collect()),
            leaf(
                (0..8)
                    .map(|triangle| SupportTriangleRef::Foundation {
                        owner: 19,
                        triangle,
                    })
                    .collect(),
            ),
        ]);
        let json = serde_json::to_value(&index).unwrap();
        assert!(json.is_array());
        let mut bytes = Vec::new();
        ciborium::into_writer(&index, &mut bytes).unwrap();
        let restored: SupportQueryIndex = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(restored, index);
        assert_eq!(restored.0[1].minimum.x.to_bits(), (-0.0_f32).to_bits());
        assert!(bytes.len() < serde_json::to_vec(&index).unwrap().len() / 2);
        let mut postcard = postcard::to_allocvec(&index).unwrap();
        assert_eq!(
            postcard::from_bytes::<SupportQueryIndex>(&postcard).unwrap(),
            index
        );
        postcard.pop();
        assert!(postcard::from_bytes::<SupportQueryIndex>(&postcard).is_err());
    }

    #[test]
    fn malformed_binary_index_rejects_cycles_and_out_of_range_leaves() {
        for tail in [[0, 0, 0], [1, 0, 1], [2, 0, 0]] {
            let packed = PackedIndex {
                nodes: vec![[0, 0, 0, 0, tail[0], tail[1], tail[2]]],
                triangles: vec![],
            };
            let mut bytes = Vec::new();
            ciborium::into_writer(&packed, &mut bytes).unwrap();
            assert!(ciborium::from_reader::<SupportQueryIndex, _>(bytes.as_slice()).is_err());
        }
    }
}
