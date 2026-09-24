//! Garments saved once draped and settled, worn again on other bodies without
//! simulating.
//!
//! Every cloth vertex is bound to the point of the wearer's surface nearest to
//! it when the garment settled: a body triangle, barycentric coordinates on
//! it, and the offset from that point in the surface's local frame. Wearing
//! the garment evaluates the bindings on the new wearer, so the cloth follows
//! the body's shape, slope and size while keeping its ease and folds. Cloth
//! stretched far past its settled shape, where neighbouring vertices were
//! bound to body parts that moved apart, is pulled back together, and the
//! cloth is then pushed clear of the wearer, inner garments and plate, and
//! lifted over body features that pass between its vertices.

mod packed;

use super::conform::{SewnCloth, TRANSFER_STRETCH, barycentric, unit};
use super::*;
use packed::Packed;

/// A garment draped and settled once, with the settings it was made from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettledGarment {
    /// Construction, fabric, layer and appearance. Its drape settings record
    /// how the garment was settled; wearing it never simulates again.
    pub selection: GarmentSelection,
    pub drape: SettledDrape,
}

impl SettledGarment {
    /// Save `garment`, fully settled on the body in `input`.
    pub fn capture(input: &DrapeInput, garment: &DrapedGarment) -> Result<Self> {
        Ok(Self {
            selection: input.selection.clone(),
            drape: SettledDrape::capture(input, garment)?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        self.selection.validate()?;
        self.drape.validate()
    }
}

/// The wearer mesh a drape is bound to. Bindings name its triangles, so a
/// drape is only worn on bodies of the same mesh: the same MHR level of detail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyTopology {
    pub vertices: u32,
    pub triangles: u32,
}

impl BodyTopology {
    fn of(input: &DrapeInput) -> Self {
        Self {
            vertices: input.positions.len() as u32,
            triangles: input.faces.len() as u32,
        }
    }
}

/// Where one cloth vertex sits relative to the wearer's surface.
#[derive(Clone, Copy, Debug, PartialEq)]
struct SurfaceBinding {
    triangle: u32,
    /// Weights of the triangle's second and third corners.
    barycentric: [f32; 2],
    /// Offset from the surface point along its tangent, bitangent and normal.
    offset: [f32; 3],
}

impl Packed for SurfaceBinding {
    const WORDS: usize = 6;
    fn pack(&self, words: &mut Vec<u32>) {
        words.push(self.triangle);
        words.extend(self.barycentric.map(f32::to_bits));
        words.extend(self.offset.map(f32::to_bits));
    }
    fn unpack(words: &[u32]) -> Self {
        Self {
            triangle: words[0],
            barycentric: [1, 2].map(|i| f32::from_bits(words[i])),
            offset: [3, 4, 5].map(|i| f32::from_bits(words[i])),
        }
    }
}

/// A settled garment's mesh, bound to the body surface it settled on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettledDrape {
    body: BodyTopology,
    /// Where each vertex settled, which the transfer moves from.
    #[serde(with = "packed")]
    positions: Vec<[f32; 3]>,
    #[serde(with = "packed")]
    texcoords: Vec<[f32; 2]>,
    #[serde(with = "packed")]
    faces: Vec<[u32; 3]>,
    #[serde(with = "packed")]
    bindings: Vec<SurfaceBinding>,
}

impl SettledDrape {
    /// Bind `garment`, fully settled on the body in `input`, to that body.
    pub fn capture(input: &DrapeInput, garment: &DrapedGarment) -> Result<Self> {
        ensure_settled(garment.stage)?;
        anyhow::ensure!(
            garment.texcoords.len() == garment.positions.len(),
            "the garment's material coordinates do not match its vertices"
        );
        let wearer = Wearer::new(&input.positions, &input.faces);
        let bindings = garment
            .positions
            .iter()
            .map(|&p| wearer.bind(vector(p)))
            .collect::<Result<_>>()?;
        let drape = Self {
            body: BodyTopology::of(input),
            positions: garment.positions.clone(),
            texcoords: garment.texcoords.clone(),
            faces: garment.faces.clone(),
            bindings,
        };
        drape.validate()?;
        Ok(drape)
    }

    pub fn vertex_count(&self) -> usize {
        self.bindings.len()
    }

    pub fn body(&self) -> BodyTopology {
        self.body
    }

    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            !self.bindings.is_empty() && !self.faces.is_empty(),
            "a saved drape needs cloth"
        );
        anyhow::ensure!(
            self.texcoords.len() == self.bindings.len()
                && self.positions.len() == self.bindings.len(),
            "a saved drape needs a position and material coordinates for every vertex"
        );
        let vertices = self.bindings.len() as u32;
        anyhow::ensure!(
            self.faces.iter().flatten().all(|&i| i < vertices),
            "a saved drape's triangle names a missing vertex"
        );
        anyhow::ensure!(
            self.bindings.iter().all(|binding| {
                binding.triangle < self.body.triangles
                    && binding
                        .barycentric
                        .iter()
                        .chain(&binding.offset)
                        .all(|x| x.is_finite())
            }) && self
                .texcoords
                .iter()
                .flatten()
                .chain(self.positions.iter().flatten())
                .all(|x| x.is_finite()),
            "a saved drape is bound to a missing body triangle or holds non-finite values"
        );
        Ok(())
    }

    /// Fit the drape to the wearer in `input`, clear of its inner garments and
    /// plate. Clearance problems are warnings; the garment is usable anyway.
    pub fn wear(&self, input: &DrapeInput) -> DrapeOutcome {
        let mut warnings = Vec::new();
        let result = self.fit(input, &mut warnings);
        DrapeOutcome {
            result,
            warnings,
            checkpoints: DrapeCheckpoints::new(input),
        }
    }

    fn fit(&self, input: &DrapeInput, warnings: &mut Vec<String>) -> Result<DrapedGarment> {
        let body = BodyTopology::of(input);
        anyhow::ensure!(
            body == self.body,
            "the drape was saved on a body mesh of {} vertices, but this body has {}; \
             choose the level of detail it was saved on",
            self.body.vertices,
            body.vertices
        );
        let wearer = Wearer::new(&input.positions, &input.faces);
        let placed: Vec<_> = self
            .bindings
            .iter()
            .map(|binding| array(wearer.place(binding)))
            .collect();
        let cloth = SewnCloth::new(&self.positions, &self.faces);
        let collision = super::placement::collision_surface(input);
        let clearance = super::drape::body_clearance(input);
        let held = cloth.limit_stretch(&self.positions, &placed, TRANSFER_STRETCH);
        let positions = cloth.keep_out(&held, &collision, clearance);
        let selection = &input.selection;
        let mut garment = DrapedGarment {
            form: selection.form(),
            name: selection.name.clone(),
            fabric: selection.fabric,
            texcoords: self.texcoords.clone(),
            positions,
            normals: Vec::new(),
            faces: self.faces.clone(),
            indices: Vec::new(),
            weights: Vec::new(),
            stage: DrapeStage::Worn,
        };
        garment.normals = garment.normals_for(&garment.positions);
        (garment.indices, garment.weights) = transfer_skin(input, &garment.positions)?;
        warnings.extend(garment.contact_issues(&collision));
        Ok(garment)
    }
}

/// Garments are saved once settling has run to its last step.
fn ensure_settled(stage: DrapeStage) -> Result<()> {
    match stage {
        DrapeStage::Settling { step, of } if step == of => Ok(()),
        DrapeStage::Worn => Ok(()),
        stage => bail!("the garment has not settled yet ({stage})"),
    }
}

/// The wearer's surface, with smooth normals for continuous local frames.
struct Wearer<'a> {
    tree: fabelgeist_bvh::TriangleBvh,
    faces: &'a [[u32; 3]],
    normals: Vec<[f32; 3]>,
}

impl<'a> Wearer<'a> {
    fn new(positions: &[[f32; 3]], faces: &'a [[u32; 3]]) -> Self {
        Self {
            tree: fabelgeist_bvh::TriangleBvh::new(
                positions.iter().copied().map(vector).collect(),
                faces.to_vec(),
            ),
            faces,
            normals: normals(positions, faces),
        }
    }

    fn bind(&self, point: Vec3) -> Result<SurfaceBinding> {
        let (triangle, closest, _) = self
            .tree
            .closest_point(point, f32::MAX)
            .context("the wearer has no surface to bind the garment to")?;
        let (a, b, c) = self.tree.triangle(triangle);
        let [_, v, w] = barycentric(a, b, c, closest);
        let barycentric = [v, w];
        let (origin, frame) = self.frame(triangle, barycentric);
        let offset = point - origin;
        Ok(SurfaceBinding {
            triangle,
            barycentric,
            offset: frame.map(|axis| axis.dot(offset)),
        })
    }

    fn place(&self, binding: &SurfaceBinding) -> Vec3 {
        let (origin, frame) = self.frame(binding.triangle, binding.barycentric);
        frame
            .iter()
            .zip(binding.offset)
            .fold(origin, |point, (axis, distance)| point + *axis * distance)
    }

    /// The surface point and its tangent, bitangent and normal. The normal is
    /// interpolated from the corners, so neighbouring triangles agree on it.
    fn frame(&self, triangle: u32, [v, w]: [f32; 2]) -> (Vec3, [Vec3; 3]) {
        let (a, b, c) = self.tree.triangle(triangle);
        let weights = [1.0 - v - w, v, w];
        let point = a * weights[0] + b * weights[1] + c * weights[2];
        let corners = self.faces[triangle as usize];
        let smooth = corners
            .iter()
            .zip(weights)
            .fold(Vec3::default(), |sum, (&i, weight)| {
                sum + vector(self.normals[i as usize]) * weight
            });
        let face = (b - a).cross(c - a);
        let normal = unit(smooth)
            .or_else(|| unit(face))
            .unwrap_or(Vec3::new(0.0, 1.0, 0.0));
        let edge = b - a;
        let tangent = unit(edge - normal * normal.dot(edge))
            .or_else(|| unit(normal.cross(Vec3::new(1.0, 0.0, 0.0))))
            .unwrap_or(Vec3::new(0.0, 0.0, 1.0));
        (point, [tangent, normal.cross(tangent), normal])
    }
}

#[cfg(test)]
mod tests;
