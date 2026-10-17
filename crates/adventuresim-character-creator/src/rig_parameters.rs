//! Checked rig ordinals at the native 32-bit GPU uniform boundary.

use anyhow::Result;
use fabelgeist_gpu::prelude::PassParameters;
use fabelgeist_rig::RigJointOrdinal;

pub(crate) trait RigPassParameters {
    fn insert_joint(&mut self, name: &'static str, joint: RigJointOrdinal) -> Result<()>;
}

impl RigPassParameters for PassParameters {
    fn insert_joint(&mut self, name: &'static str, joint: RigJointOrdinal) -> Result<()> {
        self.insert(name, u32::try_from(joint)?);
        Ok(())
    }
}
