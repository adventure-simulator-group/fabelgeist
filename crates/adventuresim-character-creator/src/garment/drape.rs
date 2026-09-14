use super::stages::DrapeStart;
use super::*;

const STEP_SECONDS: f32 = 1.0 / 60.0;

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
    let result = run(&input, start, &mut checkpoints, cancel, &mut preview);
    DrapeOutcome {
        result,
        checkpoints,
    }
}

fn run(
    input: &DrapeInput,
    start: DrapeStart,
    checkpoints: &mut DrapeCheckpoints,
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
        DrapeStart::ArmorFit { sewn, settled } => {
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
    if let Some(armor) = &input.armor {
        output.finish_armor(
            armor,
            &collision,
            body_clearance(input),
            &input.selection.drape.armor_fit,
        )?;
    }
    (output.indices, output.weights) = transfer_skin(input, &output.positions)?;
    output.validate_contacts(&collision)?;
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
    let build = if input.selection.preset.is_fitted() {
        fabelgeist_garment_fit::GarmentBuild {
            mesh: super::fitted::coif(input, &input.selection.coif, &fabric)
                .context("fitting the coif to the character")?,
            skipped: Vec::new(),
        }
    } else {
        let design = input.selection.design()?;
        let pattern =
            MetaGarment::new(input.selection.preset.label(), &body, &design).assembly();
        build_garment(&pattern, &settings, &fabric)?
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
                    let steps = stages.sewing.steps;
                    for step in 1..=steps {
                        cancelled()?;
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
                    let sewn = fit.positions().await?;
                    checkpoints.record_sewn(sewn.clone());
                    sewn
                }
            };
            // Settling starts at rest from the sewn shape.
            let sewn: Vec<_> = sewn.into_iter().map(vector).collect();
            fit.cloth.particles.write_positions(&fit.context, &sewn)?;
            fit.cloth.outer_layer = input
                .armor
                .as_ref()
                .map(|armor| super::armor::outer_layer(armor, fabric))
                .transpose()?;
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

fn fit_settings(input: &DrapeInput, body: &Body) -> FitSettings {
    FitSettings {
        resolution_cm: input.selection.resolution_cm,
        body_height_cm: body.get("height") as f32,
        // A mail underlayer needs less ease against the wearer than loose cloth.
        body_offset_cm: input.selection.fabric.body_ease_cm(input.armor.as_ref()),
        ..Default::default()
    }
}

/// Wearer clearance for the cloth mid-surface: ease plus half the thickness.
fn body_clearance(input: &DrapeInput) -> f32 {
    input.selection.fabric.body_ease_cm(input.armor.as_ref()) * fabelgeist_garment_fit::CM_TO_M
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
