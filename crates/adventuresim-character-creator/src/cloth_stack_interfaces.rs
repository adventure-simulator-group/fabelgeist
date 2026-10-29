//! Adjacent lining sheets share buried interfaces only by exact source ancestry.
//! Native positions are canonical metres. Visible faces and all edge walls stay.
use super::BodyClothPlan;
use crate::cloth_sewn_interfaces::{MAXIMUM_SEWN_INTERFACE_GAP, retain_faces, validate_endpoints};
use crate::device_fit::DevicePiece;
use crate::surface_cut::POINT_WORDS;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_armor::{ArmorComponent, BuiltPart, PlateFace};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PointIdentity(Vec<SourceWeight>);

/// Canonical native source ordinals and dimensionless f32 weight encodings.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceWeight {
    vertex: u32,
    weight_bits: u32,
}

struct MatchedInterface {
    parent: usize,
    points: [usize; 3],
}

struct SheetIdentities {
    points: Vec<PointIdentity>,
}

struct StackInterfaces {
    retained: Vec<bool>,
    skin_owners: BTreeMap<usize, usize>,
    claimed: BTreeSet<usize>,
}

impl BodyClothPlan {
    pub(crate) fn join_layer_interfaces(&self, piece: &mut DevicePiece) -> Result<()> {
        validate_endpoints(piece)?;
        ensure!(
            piece.base.components.len() == self.layers.len(),
            "cloth stack lost material ownership"
        );
        let sheets = self.sheet_identities()?;
        let mut interfaces = StackInterfaces {
            retained: vec![true; piece.base.indices.len() / 3],
            skin_owners: BTreeMap::new(),
            claimed: BTreeSet::new(),
        };
        for upper in 1..self.layers.len() {
            let lower = upper - 1;
            let contact = self.layers[upper].offset_metres - self.layers[upper].thickness_metres;
            if (contact - self.layers[lower].offset_metres).abs()
                > MAXIMUM_SEWN_INTERFACE_GAP.metres()
            {
                continue;
            }
            interfaces.collect(
                piece,
                &piece.base.components[lower],
                &sheets[lower],
                &piece.base.components[upper],
                &sheets[upper],
            )?;
        }
        interfaces.apply(piece)?;
        Ok(())
    }

    /// The face shell and native lining stack use the same frozen torso field.
    /// Call before material partitioning changes the raw carrier alias frame.
    pub(crate) fn join_gown_interface(
        &self,
        piece: &mut DevicePiece,
        gown: &crate::skirt_join::SkirtJoinPlan,
    ) -> Result<()> {
        let Some(last_layer) = self.layers.last() else {
            return Ok(());
        };
        let contact = gown.offset_metres - gown.face_thickness.metres();
        if (contact - last_layer.offset_metres).abs() > MAXIMUM_SEWN_INTERFACE_GAP.metres() {
            return Ok(());
        }
        validate_endpoints(piece)?;
        let lower = piece
            .base
            .components
            .last()
            .context("gown has no lining sheet")?;
        let upper = piece
            .base
            .components
            .first()
            .context("gown has no face shell")?;
        let mut points = vec![PointIdentity(Vec::new()); upper.vertices.len()];
        let rows = identities(&gown.table);
        let first = gown.patch.carrier_point_count();
        let destination = points
            .get_mut(first..first + rows.len())
            .context("gown source rows exceed its raw face carrier")?;
        destination.clone_from_slice(&rows);
        let upper_sheet = SheetIdentities { points };
        let sheets = self.sheet_identities()?;
        let lower_sheet = sheets
            .last()
            .context("gown lining has no source identities")?;
        let mut interfaces = StackInterfaces {
            retained: vec![true; piece.base.indices.len() / 3],
            skin_owners: BTreeMap::new(),
            claimed: BTreeSet::new(),
        };
        interfaces.collect(piece, lower, lower_sheet, upper, &upper_sheet)?;
        interfaces.apply(piece)?;
        Ok(())
    }

    fn sheet_identities(&self) -> Result<Vec<SheetIdentities>> {
        let mut first = 0;
        let mut sheets = Vec::new();
        for layer in &self.layers {
            let end = first + layer.point_count as usize * POINT_WORDS;
            let table = self
                .table
                .get(first..end)
                .context("partial cloth stack source table")?;
            let points = identities(table);
            sheets.push(SheetIdentities { points });
            first = end;
        }
        ensure!(
            first == self.table.len(),
            "cloth stack has unused source rows"
        );
        Ok(sheets)
    }
}

impl StackInterfaces {
    fn apply(self, piece: &mut DevicePiece) -> Result<()> {
        for (vertex, &alias) in piece.skin_aliases.iter().enumerate() {
            if let Some(&owner) = self.skin_owners.get(&alias) {
                piece
                    .skin
                    .joint_indices
                    .set_vertex(vertex, piece.skin.joint_indices.vertex(owner))?;
                piece.skin.joint_weights[vertex] = piece.skin.joint_weights[owner];
            }
        }
        retain_faces(&mut piece.base, &self.retained);
        for part in &mut piece.endpoints {
            retain_faces(part, &self.retained);
        }
        Ok(())
    }

    fn collect(
        &mut self,
        piece: &DevicePiece,
        lower: &ArmorComponent,
        lower_sheet: &SheetIdentities,
        upper: &ArmorComponent,
        upper_sheet: &SheetIdentities,
    ) -> Result<()> {
        let mut parents = BTreeMap::<Vec<PointIdentity>, Vec<usize>>::new();
        for face in lower.indices.start / 3..lower.indices.end / 3 {
            if piece.base.faces[face] == PlateFace::Outer {
                parents
                    .entry(face_identity(piece, face, lower, lower_sheet)?)
                    .or_default()
                    .push(face);
            }
        }
        for face in upper.indices.start / 3..upper.indices.end / 3 {
            if piece.base.faces[face] != PlateFace::Inner {
                continue;
            }
            let key = face_identity(piece, face, upper, upper_sheet)?;
            let Some(candidates) = parents.get(&key) else {
                continue;
            };
            let upper_vertices = vertices(&piece.base, face);
            let mut matches = Vec::new();
            for &parent in candidates {
                let lower_vertices = vertices(&piece.base, parent);
                let mapped = upper_vertices
                    .map(|vertex| {
                        let identity = vertex_identity(piece, vertex, upper, upper_sheet)?;
                        lower_vertices
                            .iter()
                            .copied()
                            .find(|&v| {
                                vertex_identity(piece, v, lower, lower_sheet)
                                    .is_ok_and(|other| other == identity)
                            })
                            .context("stack interface lost a matching source point")
                    })
                    .into_iter()
                    .collect::<Result<Vec<_>>>()?;
                let mapped: [usize; 3] = mapped
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("partial stack facet"))?;
                if !coincident(&piece.base, upper_vertices, mapped, lower_vertices) {
                    continue;
                }
                ensure!(
                    piece.endpoints.iter().all(|part| coincident(
                        part,
                        upper_vertices,
                        mapped,
                        lower_vertices
                    )),
                    "sewn cloth stack interface separated on a morph"
                );
                matches.push(MatchedInterface {
                    parent,
                    points: mapped,
                });
            }
            ensure!(matches.len() <= 1, "ambiguous sewn cloth stack interface");
            let Some(MatchedInterface {
                parent,
                points: mapped,
            }) = matches.pop()
            else {
                continue;
            };
            ensure!(
                self.claimed.insert(parent),
                "two cloth sheets claim the same buried interface"
            );
            self.retained[parent] = false;
            self.retained[face] = false;
            for (vertex, owner) in upper_vertices.into_iter().zip(mapped) {
                let alias = piece.skin_aliases[vertex];
                if let Some(previous) = self.skin_owners.insert(alias, owner) {
                    ensure!(
                        piece.skin.joint_indices.vertex(previous)
                            == piece.skin.joint_indices.vertex(owner)
                            && piece.skin.joint_weights[previous]
                                == piece.skin.joint_weights[owner],
                        "cloth stack interface has conflicting skin ownership"
                    );
                }
            }
        }
        Ok(())
    }
}

fn vertex_identity<'a>(
    piece: &DevicePiece,
    vertex: usize,
    component: &ArmorComponent,
    sheet: &'a SheetIdentities,
) -> Result<&'a PointIdentity> {
    let ordinal = piece.skin_aliases[vertex]
        .checked_sub(component.vertices.start)
        .context("cloth stack alias belongs to a different sheet")?;
    sheet
        .points
        .get(ordinal)
        .context("cloth stack alias has no source identity")
}

fn face_identity(
    piece: &DevicePiece,
    face: usize,
    component: &ArmorComponent,
    sheet: &SheetIdentities,
) -> Result<Vec<PointIdentity>> {
    let mut key = vertices(&piece.base, face)
        .into_iter()
        .map(|v| vertex_identity(piece, v, component, sheet).cloned())
        .collect::<Result<Vec<_>>>()?;
    key.sort();
    Ok(key)
}

fn vertices(part: &BuiltPart, face: usize) -> [usize; 3] {
    [
        part.indices[face * 3] as usize,
        part.indices[face * 3 + 1] as usize,
        part.indices[face * 3 + 2] as usize,
    ]
}

fn coincident(part: &BuiltPart, first: [usize; 3], second: [usize; 3], parent: [usize; 3]) -> bool {
    let a = first.map(|v| Vec3::from_array(part.positions[v]));
    let b = second.map(|v| Vec3::from_array(part.positions[v]));
    let normal_points = parent.map(|v| Vec3::from_array(part.positions[v]));
    (a[1] - a[0])
        .cross(a[2] - a[0])
        .dot((normal_points[1] - normal_points[0]).cross(normal_points[2] - normal_points[0]))
        < 0.
        && a.into_iter()
            .zip(b)
            .all(|(p, q)| p.distance(q) <= MAXIMUM_SEWN_INTERFACE_GAP.metres())
}

fn identities(table: &[u32]) -> Vec<PointIdentity> {
    table
        .as_chunks::<POINT_WORDS>()
        .0
        .iter()
        .map(|row| {
            let mut terms = BTreeMap::<u32, f32>::new();
            for corner in 0..3 {
                let weight = f32::from_bits(row[6 + corner]);
                if weight != 0. {
                    *terms.entry(row[corner]).or_default() += weight;
                }
            }
            PointIdentity(
                terms
                    .into_iter()
                    .map(|(v, w)| SourceWeight {
                        vertex: v,
                        weight_bits: w.to_bits(),
                    })
                    .collect(),
            )
        })
        .collect()
}
