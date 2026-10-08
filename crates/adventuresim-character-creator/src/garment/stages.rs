//! Solver, contact and preview settings for each drape stage, and the stage
//! checkpoints that let a changed stage re-run without repeating earlier ones.
mod damping_serde;

use super::*;
use anyhow::ensure;
use fabelgeist_shell::DampingRate;
use std::ops::RangeInclusive;

/// Downward acceleration of a garment settling on the wearer, m/s².
const STANDARD_GRAVITY: f32 = 9.81;
/// Heavy drag while seams close, so panels meet without overshooting.
const SEWING_DAMPING_PER_SECOND: DampingRate = DampingRate::per_second(8.0);

/// Solver and host contact settings for one simulated stage.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct StageSettings {
    pub steps: u32,
    pub substeps: u32,
    /// Constraint sweeps within each substep.
    pub iterations: u32,
    /// Downward acceleration, m/s².
    pub gravity: f32,
    /// Exponential velocity drag, per second.
    #[serde(with = "damping_serde")]
    pub damping: DampingRate,
    /// GPU particle self-collision and swept host cloth contacts.
    pub self_collision: bool,
    /// GPU substeps swept by one host contact projection. Zero leaves contact
    /// to the GPU body collider and self-collision kernels.
    pub host_contact_interval: u32,
    /// Projection sweeps in one host contact solve.
    pub host_contact_iterations: u32,
    /// Also sweep cloth against the wearer's triangles on the host. The GPU
    /// body collider runs either way.
    pub host_body_contacts: bool,
}

impl StageSettings {
    pub const STEPS: RangeInclusive<u32> = 0..=600;
    pub const SUBSTEPS: RangeInclusive<u32> = 1..=32;
    pub const ITERATIONS: RangeInclusive<u32> = 1..=8;
    pub const GRAVITY: RangeInclusive<f32> = 0.0..=30.0;
    pub const DAMPING: RangeInclusive<DampingRate> =
        DampingRate::per_second(0.0)..=DampingRate::per_second(20.0);
    pub const CONTACT_ITERATIONS: RangeInclusive<u32> = 1..=8;

    fn validate(&self, stage: &str) -> Result<()> {
        ensure!(
            Self::STEPS.contains(&self.steps),
            "{stage} steps must be within {:?}",
            Self::STEPS
        );
        ensure!(
            Self::SUBSTEPS.contains(&self.substeps),
            "{stage} substeps must be within {:?}",
            Self::SUBSTEPS
        );
        ensure!(
            Self::ITERATIONS.contains(&self.iterations),
            "{stage} constraint iterations must be within {:?}",
            Self::ITERATIONS
        );
        ensure!(
            self.gravity.is_finite() && Self::GRAVITY.contains(&self.gravity),
            "{stage} gravity must be within {:?} m/s²",
            Self::GRAVITY
        );
        ensure!(
            self.damping.is_finite() && Self::DAMPING.contains(&self.damping),
            "{stage} damping must be within {:?} per second",
            Self::DAMPING
        );
        ensure!(
            self.host_contact_interval <= self.substeps,
            "{stage} host contacts cannot be spaced further apart than one step"
        );
        ensure!(
            Self::CONTACT_ITERATIONS.contains(&self.host_contact_iterations),
            "{stage} host contact iterations must be within {:?}",
            Self::CONTACT_ITERATIONS
        );
        Ok(())
    }

    /// Configure the solver and host contacts for this stage.
    pub(super) fn apply(
        &self,
        fit: &mut Fit,
        body: &fabelgeist_bvh::TriangleBvh,
        body_clearance: f32,
    ) -> Result<()> {
        let solver = &mut fit.solver.settings;
        solver.substeps = self.substeps;
        solver.iterations = self.iterations;
        solver.gravity = Vec3::new(0.0, -self.gravity, 0.0);
        solver.damping = self.damping;
        fit.cloth.self_collision.enabled = self.self_collision;
        fit.cloth.host_contacts = fabelgeist_shell::HostContactSchedule {
            interval_substeps: self.host_contact_interval,
            iterations: self.host_contact_iterations,
            ..fabelgeist_shell::HostContactSchedule::default()
        };
        if self.host_body_contacts {
            fit.cloth
                .set_collision_surface(&body.positions, &body.triangles, body_clearance);
        } else {
            fit.cloth.set_collision_surface(&[], &[], 0.0);
        }
        Ok(())
    }
}

/// Every drape stage's settings, plus how often previews are published.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DrapeSettings {
    pub sewing: StageSettings,
    pub settling: StageSettings,
    /// Share of the gap between settled cloth and the body parts it dresses
    /// that is closed after settling, without stretching the cloth: 0 leaves
    /// it as it hangs, 1 draws it as close as its cut allows.
    pub body_fit: f32,
    /// Simulation steps between preview snapshots.
    pub preview_interval: u32,
}

impl DrapeSettings {
    pub const PREVIEW_INTERVAL: RangeInclusive<u32> = 1..=60;
    pub const BODY_FIT: RangeInclusive<f32> = 0.0..=1.0;
    /// Close most of the gap, so cloth follows the body without clinging.
    const DEFAULT_BODY_FIT: f32 = 0.7;

    pub fn for_fabric(fabric: Fabric) -> Self {
        let substeps = FitSettings::default().substeps;
        let sewing = StageSettings {
            steps: 60,
            substeps,
            iterations: 1,
            gravity: 0.0,
            damping: SEWING_DAMPING_PER_SECOND,
            self_collision: true,
            host_contact_interval: substeps,
            host_contact_iterations: 4,
            // The GPU collider tests particle spheres only; a cloth triangle can
            // pass through body triangles between them without swept contacts.
            host_body_contacts: true,
        };
        Self {
            sewing,
            settling: StageSettings {
                steps: 60,
                gravity: STANDARD_GRAVITY,
                damping: fabric.damping,
                ..sewing
            },
            body_fit: Self::DEFAULT_BODY_FIT,
            preview_interval: 1,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.sewing.validate("sewing")?;
        self.settling.validate("settling")?;
        ensure!(
            self.body_fit.is_finite() && Self::BODY_FIT.contains(&self.body_fit),
            "body fit must be within {:?}",
            Self::BODY_FIT
        );
        ensure!(
            Self::PREVIEW_INTERVAL.contains(&self.preview_interval),
            "preview interval must be within {:?} steps",
            Self::PREVIEW_INTERVAL
        );
        Ok(())
    }
}

/// Garment state at the completed stage boundaries of one drape, with the
/// input that produced it.
#[derive(Clone)]
pub struct DrapeCheckpoints {
    input: DrapeInput,
    sewn: Option<Vec<[f32; 3]>>,
    settled: Option<DrapedGarment>,
}

/// Where a drape begins, given the previous drape's checkpoints.
pub(super) enum DrapeStart {
    Placement,
    Settling {
        sewn: Vec<[f32; 3]>,
    },
    Finish {
        sewn: Vec<[f32; 3]>,
        settled: DrapedGarment,
    },
}

impl DrapeCheckpoints {
    pub(super) fn new(input: &DrapeInput) -> Self {
        Self {
            input: input.clone(),
            sewn: None,
            settled: None,
        }
    }

    pub(super) fn record_sewn(&mut self, sewn: Vec<[f32; 3]>) {
        self.sewn = Some(sewn);
    }

    pub(super) fn record_settled(&mut self, settled: DrapedGarment) {
        self.settled = Some(settled);
    }

    /// Re-run from the earliest stage whose inputs changed. Surface appearance
    /// is not a drape input, so a material edit only repeats the finish.
    pub(super) fn start_for(previous: Option<&Self>, input: &DrapeInput) -> DrapeStart {
        let Some(previous) = previous else {
            return DrapeStart::Placement;
        };
        let (before, after) = (&previous.input.selection.drape, &input.selection.drape);
        let Some(sewn) = &previous.sewn else {
            return DrapeStart::Placement;
        };
        if !previous.input.same_setup(input) || before.sewing != after.sewing {
            return DrapeStart::Placement;
        }
        let sewn = sewn.clone();
        match &previous.settled {
            Some(settled) if before.settling == after.settling => DrapeStart::Finish {
                sewn,
                settled: settled.clone(),
            },
            _ => DrapeStart::Settling { sewn },
        }
    }
}

impl DrapeInput {
    /// Whether placement and sewing would receive identical inputs.
    fn same_setup(&self, other: &Self) -> bool {
        self.positions == other.positions
            && self.faces == other.faces
            && self.names == other.names
            && self.joints == other.joints
            && self.indices == other.indices
            && self.weights == other.weights
            && self.selection.construction == other.selection.construction
            && self.selection.fabric == other.selection.fabric
            && self.selection.resolution_cm == other.selection.resolution_cm
            && self.obstacles.len() == other.obstacles.len()
            && self
                .obstacles
                .iter()
                .zip(&other.obstacles)
                .all(|(a, b)| a.positions == b.positions && a.faces == b.faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> DrapeInput {
        DrapeInput {
            under_plate: None,
            selection: GarmentSelection::chainmail(),
            settled: None,
            obstacles: vec![],
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![[0, 1, 2]],
            names: vec![],
            joints: vec![],
            indices: vec![[0; 8]; 3],
            weights: vec![[1., 0., 0., 0., 0., 0., 0., 0.]; 3],
        }
    }

    fn completed(input: &DrapeInput) -> DrapeCheckpoints {
        let mut checkpoints = DrapeCheckpoints::new(input);
        checkpoints.record_sewn(vec![[0.0; 3]]);
        checkpoints.record_settled(DrapedGarment {
            form: input.selection.form(),
            name: "settled".into(),
            fabric: input.selection.fabric,
            texcoords: vec![],
            positions: vec![[0.0; 3]],
            normals: vec![],
            faces: vec![],
            indices: vec![],
            weights: vec![],
            stage: DrapeStage::Settling { step: 1, of: 1 },
        });
        checkpoints
    }

    #[test]
    fn a_changed_stage_reruns_from_that_stage_and_reuses_earlier_ones() {
        let original = input();
        let previous = completed(&original);
        let start = |edit: &dyn Fn(&mut DrapeInput)| {
            let mut next = original.clone();
            edit(&mut next);
            DrapeCheckpoints::start_for(Some(&previous), &next)
        };
        assert!(matches!(start(&|_| {}), DrapeStart::Finish { .. }));
        assert!(matches!(
            start(&|i| i.selection.mail.roughness = 0.9),
            DrapeStart::Finish { .. }
        ));
        assert!(matches!(
            start(&|i| i.selection.drape.settling.steps = 90),
            DrapeStart::Settling { .. }
        ));
        assert!(matches!(
            start(&|i| i.selection.drape.sewing.damping = DampingRate::per_second(2.0)),
            DrapeStart::Placement
        ));
        assert!(matches!(
            start(&|i| i.positions[0][0] = 0.5),
            DrapeStart::Placement
        ));
        assert!(matches!(
            DrapeCheckpoints::start_for(None, &original),
            DrapeStart::Placement
        ));
    }

    #[test]
    fn an_unfinished_drape_resumes_only_from_its_completed_stages() {
        let original = input();
        let mut sewn_only = DrapeCheckpoints::new(&original);
        sewn_only.record_sewn(vec![[0.0; 3]]);
        assert!(matches!(
            DrapeCheckpoints::start_for(Some(&sewn_only), &original),
            DrapeStart::Settling { .. }
        ));
        let nothing = DrapeCheckpoints::new(&original);
        assert!(matches!(
            DrapeCheckpoints::start_for(Some(&nothing), &original),
            DrapeStart::Placement
        ));
    }

    #[test]
    fn stage_settings_reject_contacts_spaced_beyond_one_step() {
        let mut settings = DrapeSettings::for_fabric(Fabric::CHAINMAIL);
        assert!(settings.validate().is_ok());
        settings.settling.host_contact_interval = settings.settling.substeps + 1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn damping_scalar_codec_preserves_zero_sign_and_local_rejection_order() {
        let mut settings = DrapeSettings::for_fabric(Fabric::COTTON);
        for native in [-0.0, 0.0, 20.0] {
            settings.settling.damping = DampingRate::from(native);
            assert!(settings.validate().is_ok());
            let encoded = serde_json::to_string(&settings).unwrap();
            let decoded: DrapeSettings = serde_json::from_str(&encoded).unwrap();
            assert_eq!(
                f32::from(decoded.settling.damping).to_bits(),
                native.to_bits()
            );
        }
        settings.settling.damping = DampingRate::per_second(-1.0);
        assert_eq!(
            settings.validate().unwrap_err().to_string(),
            "settling damping must be within 0.0..=20.0 per second"
        );
        settings.sewing.steps = 601;
        assert_eq!(
            settings.validate().unwrap_err().to_string(),
            "sewing steps must be within 0..=600"
        );
        settings.sewing.steps = 60;
        settings.settling.damping = DampingRate::from(f32::NAN);
        let encoded = serde_json::to_string(&settings).unwrap();
        assert!(encoded.contains("\"damping\":null"));
        let error = serde_json::from_str::<DrapeSettings>(&encoded).unwrap_err();
        assert!(error.to_string().contains("null, expected f32"));
    }

    #[test]
    fn applying_a_stage_retains_its_rate_in_solver_settings() -> Result<()> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                use fabelgeist_math::Vec2;
                let context = fabelgeist_gpu::prelude::WgpuContext::new_compute().await?;
                // Authored metre-space triangle with XY texture coordinates.
                let shell_mesh = fabelgeist_shell::ShellMesh::new(
                    vec![
                        Vec3::default(),
                        Vec3::new(0.1, 0.0, 0.0),
                        Vec3::new(0.0, 0.1, 0.0),
                    ],
                    vec![[0, 1, 2]],
                    Fabric::COTTON.density,
                )?;
                let build = fabelgeist_garment_fit::GarmentBuild {
                    mesh: fabelgeist_cloth::GarmentMesh {
                        positions: shell_mesh.positions.clone(),
                        triangles: shell_mesh.triangles.clone(),
                        material: vec![Vec2::default(), Vec2::new(0.1, 0.0), Vec2::new(0.0, 0.1)],
                        edges: shell_mesh.edges.clone(),
                        rest_lengths: shell_mesh.rest_lengths.clone(),
                        masses: shell_mesh.masses.clone(),
                        ..Default::default()
                    },
                    skipped: vec![],
                };
                let fit_settings = FitSettings {
                    self_collision: false,
                    gravity: false,
                    host_contact_interval: 0,
                    ..Default::default()
                };
                let mut fit = Fit::new(context, &build, Fabric::COTTON, &fit_settings)?;
                let body = fabelgeist_bvh::TriangleBvh::new(vec![], vec![]);
                let mut settings = DrapeSettings::for_fabric(Fabric::COTTON).settling;
                settings.host_body_contacts = false;
                for native in [-0.0, 2.0, f32::from_bits(0x7fc1_2345)] {
                    settings.damping = DampingRate::from(native);
                    settings.apply(&mut fit, &body, 0.0)?;
                    assert_eq!(
                        f32::from(fit.solver.settings.damping).to_bits(),
                        native.to_bits()
                    );
                }
                Ok(())
            })
    }
}
