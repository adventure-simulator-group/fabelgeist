//! Collision storage and native sample addresses for one simulated prefix.

use super::collision_progress::CollisionProgress;
use super::{SimSettings, StrandLayout, storage};
use crate::gpu::error::{HairGpuError, Result};
use crate::parameters::StrandCount;
use fabelgeist_compute::{PrefixSumScratch, ScanElementCount};
use fabelgeist_gpu::prelude::{Buffer, BufferElementCount, InvocationCount, WgpuContext};
use fabelgeist_math::Vec3;

/// The grid the strands' samples are sorted into to find their neighbours:
/// cells as wide as a sample's reach, round the groom in the head's frame
/// (samples outside are kept in the nearest cell), and how many samples
/// each holds, scanned into where each cell's begin.
pub(super) struct CollisionCells {
    pub lo: Vec3,
    pub cell: f32,
    pub dims: [u32; 3],
    pub counts: Buffer,
    pub scan: PrefixSumScratch,
}

/// Private native collision cardinalities; all expanded addresses fit u32.
pub(super) struct CollisionLayout {
    strands: StrandLayout,
    per_segment: u32,
}

/// Work arrays allocated together for the admitted colliding prefix.
pub(super) struct CollisionBuffers {
    pub layout: CollisionLayout,
    pub cells: CollisionCells,
    pub samples: [Buffer; 2],
    pub progress: CollisionProgress,
    pub sample_data: Buffer,
    pub radius: f32,
    pub sample_slots: Buffer,
    pub sorted: Buffer,
    pub sorted_index: Buffer,
    pub asks: Buffer,
}

impl CollisionLayout {
    fn new(strands: StrandLayout, per_segment: u32) -> Result<Self> {
        // sim_samples stores the strand ordinal in f32. Its final ordinal
        // must stay within binary32's consecutive-integer range, inclusive.
        let maximum = (1usize << f32::MANTISSA_DIGITS) + 1;
        if strands.strands().count() > maximum {
            return Err(HairGpuError::CollisionStrandIdentity {
                requested: strands.strands(),
            });
        }
        let layout = Self { strands, per_segment };
        BufferElementCount::from(strands.native_strands() * (strands.native_points() - 1))
            .checked_mul(per_segment)
            .and_then(|samples| samples.checked_mul(8))
            .ok_or(HairGpuError::CollisionSamples { strands, per_segment })?;
        Ok(layout)
    }

    pub fn strands(&self) -> StrandLayout {
        self.strands
    }

    pub fn per_segment(&self) -> u32 {
        self.per_segment
    }

    /// Largest native array is eight vec4 records per sample (sim_sort_samples).
    pub fn samples(&self) -> BufferElementCount {
        BufferElementCount::from(
            self.strands.native_strands() * (self.strands.native_points() - 1) * self.per_segment,
        )
    }

    pub fn sample_invocations(&self) -> InvocationCount {
        self.samples().into()
    }
}

impl CollisionBuffers {
    pub fn new(
        context: &WgpuContext,
        simulated: StrandLayout,
        settings: SimSettings,
        lo: Vec3,
        hi: Vec3,
        longest: f32,
    ) -> Result<Self> {
        // A populated groom keeps at least one collider; an empty one has none.
        let count = simulated.strands().count();
        let colliders = settings.collision_strands.min(count).max(1.min(count));
        let strands = simulated.prefix(StrandCount::from_count(colliders))?;
        let radius = if settings.collision_radius > 0.0 {
            settings.collision_radius
        } else {
            (0.6 * (0.06 / colliders as f32).sqrt()).clamp(0.002, 0.008)
        };
        let per_segment = ((longest / (2.0 * radius)).ceil() as u32).clamp(1, 16);
        let layout = CollisionLayout::new(strands, per_segment)?;
        let cells = CollisionCells::new(context, lo, hi, radius * 1.75)?;
        // Logical zero stays zero; storage() alone supplies the native binding floor.
        let sample_bytes = usize::from(layout.samples()) as u64 * size_of::<[f32; 4]>() as u64;
        Ok(Self {
            layout,
            cells,
            samples: [
                storage(context, sample_bytes.into(), "hair sim samples".into())?,
                storage(context, sample_bytes.into(), "hair sim samples before".into())?,
            ],
            progress: CollisionProgress::default(),
            sample_data: storage(context, (sample_bytes * 5).into(), "hair sim sample segments".into())?,
            radius,
            sample_slots: storage(context, (sample_bytes / 2).into(), "hair sim sample cells".into())?,
            sorted: storage(context, (sample_bytes * 8).into(), "hair sim sorted samples".into())?,
            sorted_index: storage(context, (sample_bytes / 4).into(), "hair sim sorted samples' order".into())?,
            asks: storage(context, (sample_bytes * 2).into(), "hair sim collision asks".into())?,
        })
    }
}

impl CollisionCells {
    fn new(context: &WgpuContext, lo: Vec3, hi: Vec3, reach: f32) -> Result<Self> {
        // Samples two radii apart along segments need this neighbourhood reach.
        let mut cell = reach;
        let cell_dims = |cell: f32| {
            let extent = hi - lo;
            [extent.x, extent.y, extent.z].map(|e| ((e / cell).ceil() as u32).max(1))
        };
        while cell_dims(cell).iter().map(|&d| d as u64).product::<u64>() > 1 << 23 {
            cell *= 1.25;
        }
        let dims = cell_dims(cell);
        let cell_count = dims.iter().product::<u32>();
        Ok(Self {
            lo,
            cell,
            dims,
            counts: storage(context, ((cell_count as u64 + 1) * 4).into(), "hair sim collision cells".into())?,
            scan: PrefixSumScratch::new(context, ScanElementCount::from_elements(cell_count + 1))?,
        })
    }
}
impl CollisionCells {
    pub fn cells(&self) -> u32 {
        self.dims[0] * self.dims[1] * self.dims[2]
    }
}

