use super::stages::DrapeStart;
use super::*;

const STEP_SECONDS: f32 = 1.0 / 60.0;
/// Share of the sewing steps over which the seams are drawn shut; the rest
/// hold them closed. Near-rigid zero-length seams would otherwise snap shut in
/// the first few frames, cutting sleeves and other tubes straight through the
/// limb inside them instead of wrapping them round it.
const SEAM_CLOSING_SHARE: f32 = 0.75;

/// Drape one garment, resuming from the previous drape's checkpoints when only
/// later stages changed. Completed stages are returned even when draping fails.
pub fn drape(
    input: DrapeInput,
    previous: Option<&DrapeCheckpoints>,
    cancel: &AtomicBool,
    mut preview: impl FnMut(DrapedGarment),
) -> DrapeOutcome {
    let start = DrapeCheckpoints::start_for(previous, &input);
    let mut checkpoints = DrapeCheckpoints::new(&input);
    let mut warnings = Vec::new();
    let result = run(
        &input,
        start,
        &mut checkpoints,
        &mut warnings,
        cancel,
        &mut preview,
    );
    DrapeOutcome {
        result,
        warnings,
        checkpoints,
    }
}

fn run(
    input: &DrapeInput,
    start: DrapeStart,
    checkpoints: &mut DrapeCheckpoints,
    warnings: &mut Vec<String>,
    cancel: &AtomicBool,
    preview: &mut impl FnMut(DrapedGarment),
) -> Result<DrapedGarment> {
    input.selection.validate()?;
    let cancelled = || -> Result<()> {
        if cancel.load(Ordering::Relaxed) {
            bail!("Draping cancelled");
        }
        Ok(())
    };
    cancelled()?;
    let mut output = match start {
        DrapeStart::Finish { sewn, settled } => {
            checkpoints.record_sewn(sewn);
            preview(settled.clone());
            settled
        }
        DrapeStart::Settling { sewn } => {
            simulate(input, Some(sewn), checkpoints, &cancelled, preview)?
        }
        DrapeStart::Placement => simulate(input, None, checkpoints, &cancelled, preview)?,
    };
    checkpoints.record_settled(output.clone());
    cancelled()?;
    let collision = super::placement::collision_surface(input);
    // Fair the settled cloth before its last clearance pass.
    let clearance = body_clearance(input);
    super::symmetrize::symmetrize_and_relax(
        &mut output.positions,
        &output.faces,
        &collision,
        clearance,
    );
    let cloth = super::conform::SewnCloth::new(&output.positions, &output.faces);
    let fit = input.selection.drape.body_fit;
    if fit > 0.0 {
        let dressing = super::dressing::Dressing::new(input);
        output.positions = cloth.draw_in(&output.positions, &dressing, &collision, clearance, fit);
    }
    output.positions = cloth.keep_out(&output.positions, &collision, clearance);
    if let Some(under) = &input.under_plate {
        // The plate has the last word: cloth it covers lies under it.
        output.positions = cloth.tuck_under(&output.positions, under);
    }
    output.normals = output.normals_for(&output.positions);
    (output.indices, output.weights) = transfer_skin(input, &output.positions)?;
    warnings.extend(output.contact_issues(&collision));
    Ok(output)
}

/// Place, sew (unless `sewn` resumes after sewing) and settle the garment.
fn simulate(
    input: &DrapeInput,
    sewn: Option<Vec<[f32; 3]>>,
    checkpoints: &mut DrapeCheckpoints,
    cancelled: &impl Fn() -> Result<()>,
    preview: &mut impl FnMut(DrapedGarment),
) -> Result<DrapedGarment> {
    let body = measured_body(input).context("measuring the character")?;
    cancelled()?;
    let fabric = input.selection.fabric.fabric();
    let settings = fit_settings(input, &body);
    let build = match &input.selection.construction {
        Construction::Coif(coif) => fabelgeist_garment_fit::GarmentBuild {
            mesh: super::fitted::coif(input, coif, &fabric)
                .context("fitting the coif to the character")?,
            skipped: Vec::new(),
        },
        Construction::Sewn(pattern) => {
            let design = pattern.design()?;
            let pattern = MetaGarment::new(&input.selection.name, &body, &design).assembly();
            build_garment(&pattern, &settings, &fabric)?
        }
    };
    if !build.skipped.is_empty() || build.mesh.triangles.is_empty() {
        bail!(
            "garment meshing failed; skipped panels: {:?}",
            build.skipped
        );
    }
    cancelled()?;
    let stages = &input.selection.drape;
    let clearance = body_clearance(input);
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let (mut fit, collision) =
                super::placement::prepare(input, &build, &settings, fabric).await?;
            let triangles = &build.mesh.triangles;
            let surface =
                SewnSurface::new(build.mesh.positions.len(), &build.mesh.seams, triangles);
            let mut output = DrapedGarment::from_pattern(&input.selection, &build.mesh);
            let sewn = match sewn {
                Some(sewn) => sewn,
                None => {
                    output.show_unsewn(
                        fit.positions().await?,
                        triangles,
                        &collision,
                        DrapeStage::Placed,
                    )?;
                    preview(output.clone());
                    // Close the seams before gravity can pull the still-separated
                    // panels below their supporting shoulders or waistband.
                    stages.sewing.apply(&mut fit, &collision, clearance)?;
                    let seams = SeamClosure::new(&fit, &build.mesh, &fit.positions().await?);
                    let steps = stages.sewing.steps;
                    let closing = (steps as f32 * SEAM_CLOSING_SHARE).max(1.0);
                    for step in 1..=steps {
                        cancelled()?;
                        let open = (1.0 - step as f32 / closing).max(0.0);
                        seams.hold_open(&mut fit, open)?;
                        fit.step(STEP_SECONDS).await?;
                        if step % stages.preview_interval == 0 || step == steps {
                            let stage = DrapeStage::Sewing { step, of: steps };
                            output.show_unsewn(
                                fit.positions().await?,
                                triangles,
                                &collision,
                                stage,
                            )?;
                            preview(output.clone());
                        }
                    }
                    seams.shut(&mut fit)?;
                    let sewn = fit.positions().await?;
                    checkpoints.record_sewn(sewn.clone());
                    sewn
                }
            };
            // Settling starts at rest from the sewn shape.
            let sewn: Vec<_> = sewn.into_iter().map(vector).collect();
            fit.cloth.particles.write_positions(&fit.context, &sewn)?;
            stages.settling.apply(&mut fit, &collision, clearance)?;
            let steps = stages.settling.steps;
            for step in 0..=steps {
                cancelled()?;
                if step > 0 {
                    fit.step(STEP_SECONDS).await?;
                }
                if step % stages.preview_interval == 0 || step == steps {
                    let stage = DrapeStage::Settling { step, of: steps };
                    let particles = fit.positions().await?;
                    output.show_sewn(&particles, &surface, triangles, &collision, stage)?;
                    preview(output.clone());
                }
            }
            Ok(output)
        })
}

/// Seams drawn shut over the sewing stage, from the gaps the panels were
/// placed at.
struct SeamClosure {
    gaps: Vec<f32>,
    /// Bend weights in colour order, as built and with every hinge across a
    /// seam slack. Such a hinge spans the open gap and would read it as a
    /// sharp crease, wrenching the panels round to flatten it.
    bends: Vec<f32>,
    open_bends: Vec<f32>,
}

impl SeamClosure {
    fn new(fit: &Fit, mesh: &fabelgeist_cloth::GarmentMesh, placed: &[[f32; 3]]) -> Self {
        let gaps = mesh
            .seams
            .iter()
            .map(|&[a, b]| (vector(placed[a as usize]) - vector(placed[b as usize])).length())
            .collect();
        // A hinge across a seam joins triangles whose shared edge only exists
        // once the seam's copies meet.
        let triangles: std::collections::HashSet<[u32; 3]> = mesh
            .triangles
            .iter()
            .map(|&triangle| {
                let mut sorted = triangle;
                sorted.sort_unstable();
                sorted
            })
            .collect();
        let open_bends: Vec<_> = mesh
            .bends
            .iter()
            .zip(&mesh.bend_weights)
            .map(|(bend, &weights)| {
                let [a, b, wings @ ..] = bend.particles();
                let within_the_mesh = wings.iter().all(|&wing| {
                    let mut triangle = [a, b, wing];
                    triangle.sort_unstable();
                    triangles.contains(&triangle)
                });
                if within_the_mesh { weights } else { [0.0; 8] }
            })
            .collect();
        let colour_order = |weights: &[[f32; 8]]| fit.cloth.bending.reorder(weights).concat();
        Self {
            gaps,
            bends: colour_order(&mesh.bend_weights),
            open_bends: colour_order(&open_bends),
        }
    }

    /// Hold each seam open by `open` of its placed gap.
    fn hold_open(&self, fit: &mut Fit, open: f32) -> Result<()> {
        self.set(fit, open, &self.open_bends)
    }

    /// Shut every seam and let the hinges across them bend the cloth.
    fn shut(&self, fit: &mut Fit) -> Result<()> {
        self.set(fit, 0.0, &self.bends)
    }

    fn set(&self, fit: &mut Fit, open: f32, bends: &[f32]) -> Result<()> {
        let rest: Vec<f32> = self.gaps.iter().map(|gap| gap * open).collect();
        let rest = fit.cloth.seams.reorder(&rest);
        fit.cloth
            .seams
            .attach(&fit.context, "rest_lengths", &rest)?;
        fit.cloth.bending.attach_raw(&fit.context, "weights", bends)
    }
}

fn fit_settings(input: &DrapeInput, body: &Body) -> FitSettings {
    FitSettings {
        resolution_cm: input.selection.resolution_cm,
        body_height_cm: body.get("height") as f32,
        ..Default::default()
    }
}

/// Wearer clearance for the cloth mid-surface: ease plus half the thickness.
pub(super) fn body_clearance(input: &DrapeInput) -> f32 {
    FitSettings::default().body_offset_cm * fabelgeist_garment_fit::CM_TO_M
        + input.selection.fabric.fabric().particle_radius()
}

impl DrapedGarment {
    /// Show particle positions before the seams are welded. Averaging the
    /// still-open seam copies would pull separated panels towards each other.
    fn show_unsewn(
        &mut self,
        positions: Vec<[f32; 3]>,
        triangles: &[[u32; 3]],
        body: &fabelgeist_bvh::TriangleBvh,
        stage: DrapeStage,
    ) -> Result<()> {
        anyhow::ensure!(
            positions.len() == self.texcoords.len(),
            "cloth particles do not match the garment's material vertices"
        );
        if positions.iter().flatten().any(|x| !x.is_finite()) {
            bail!("cloth simulation produced non-finite positions");
        }
        let mut faces = triangles.to_vec();
        orient_faces(&positions, &mut faces, body);
        self.normals = normals(&positions, &faces);
        self.faces = faces;
        self.positions = positions;
        self.stage = stage;
        Ok(())
    }

    /// Show the sewn surface: seam copies share one position, winding and
    /// normal, so joined panels shade continuously.
    fn show_sewn(
        &mut self,
        particles: &[[f32; 3]],
        surface: &SewnSurface,
        triangles: &[[u32; 3]],
        body: &fabelgeist_bvh::TriangleBvh,
        stage: DrapeStage,
    ) -> Result<()> {
        self.positions = surface.expand(&surface.positions(particles));
        if self.positions.iter().flatten().any(|x| !x.is_finite()) {
            bail!("cloth simulation produced non-finite positions");
        }
        let sewn = surface.positions(&self.positions);
        let mut sewn_faces = surface.faces.clone();
        orient_faces(&sewn, &mut sewn_faces, body);
        self.faces = triangles.to_vec();
        for (i, face) in sewn_faces.iter().enumerate() {
            if *face != surface.faces[i] {
                self.faces[surface.render_faces[i]].swap(1, 2);
            }
        }
        self.normals = surface.expand(&normals(&sewn, &sewn_faces));
        self.stage = stage;
        Ok(())
    }
}
