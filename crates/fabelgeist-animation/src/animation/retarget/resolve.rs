//! Turning names into joint indices.
//!
//! This is the only place in the retargeting system that looks at strings.
//! Once a profile has been resolved against a skeleton, everything downstream
//! addresses joints by index, which is what makes the retargeter itself
//! indifferent to where an animation came from.

use super::matching::RetargetMatchKey;
use super::{ChainPresence, RigJointChain};
use super::{JointRequirement, RetargetStrictness};
use super::{MissingRequiredJoint, RetargetError, RetargetProfileName, RigProfileName};
use crate::skeleton::Skeleton;
use fabelgeist_rig::{RigJointName, RigJointOrdinal};
use indexmap::IndexMap;

use super::profile::{RetargetProfile, RetargetSettings, RigProfile, RootSource};
use super::semantic::{HumanoidChain, HumanoidJoint};

/// A skeleton's joint names indexed for repeated lookups.
struct JointIndex {
    exact: IndexMap<RigJointName, RigJointOrdinal>,
    normalized: IndexMap<RetargetMatchKey, RigJointOrdinal>,
}

impl JointIndex {
    fn new(skeleton: &Skeleton) -> Self {
        let mut exact = IndexMap::new();
        let mut normalized = IndexMap::new();
        for (index, joint) in skeleton.joints.iter().enumerate() {
            let index = RigJointOrdinal::from(index);
            exact.entry(joint.name.clone()).or_insert(index);
            normalized
                .entry(RetargetMatchKey::from(&joint.name))
                .or_insert(index);
        }
        Self { exact, normalized }
    }

    fn find(&self, name: &RigJointName) -> Option<RigJointOrdinal> {
        self.exact
            .get(name)
            .or_else(|| self.normalized.get(&RetargetMatchKey::from(name)))
            .copied()
    }
}

/// A profile bound to a concrete skeleton.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedRig {
    pub profile: RigProfileName,
    /// Skeleton joint index per humanoid role that resolved.
    pub joints: IndexMap<HumanoidJoint, RigJointOrdinal>,
    /// Skeleton joint indices per chain, root-most first.
    pub chains: IndexMap<HumanoidChain, RigJointChain>,
    /// The joint carrying locomotion, if the rig has one.
    pub root: Option<RigJointOrdinal>,
    /// Optional roles the profile named but the skeleton does not have.
    pub missing: Vec<HumanoidJoint>,
    /// Skeleton joints no role or chain claims. Harmless; listed for tooling.
    pub unmapped: Vec<RigJointOrdinal>,
}

impl ResolvedRig {
    pub fn joint(&self, role: HumanoidJoint) -> Option<RigJointOrdinal> {
        self.joints.get(&role).copied()
    }

    pub fn has(&self, role: HumanoidJoint) -> bool {
        self.joints.contains_key(&role)
    }

    /// The roles this rig resolved that a later IK or contact pass would pin.
    pub fn end_effectors(&self) -> Vec<(HumanoidJoint, RigJointOrdinal)> {
        self.joints
            .iter()
            .filter(|(role, _)| role.is_end_effector())
            .map(|(role, index)| (*role, *index))
            .collect()
    }
}

impl RigProfile {
    /// Binds this profile's names to a skeleton's joints.
    ///
    /// Fails only on joints the profile marked required; everything else is
    /// reported and carried on without.
    pub fn resolve(&self, skeleton: &Skeleton) -> Result<ResolvedRig, RetargetError> {
        let index = JointIndex::new(skeleton);

        let mut joints = IndexMap::new();
        let mut missing = Vec::new();
        let mut missing_required = Vec::new();
        for (role, binding) in &self.joints {
            match binding.names.iter().find_map(|name| index.find(name)) {
                Some(joint) => {
                    joints.insert(*role, joint);
                }
                None if binding.required == JointRequirement::Required => {
                    missing_required.push(*role)
                }
                None => missing.push(*role),
            }
        }

        if !missing_required.is_empty() {
            let bindings = missing_required
                .iter()
                .map(|role: &HumanoidJoint| -> MissingRequiredJoint {
                    MissingRequiredJoint {
                        role: *role,
                        candidates: self.joints[role].names.clone(),
                    }
                })
                .collect();
            return Err(RetargetError::RequiredJointsMissing {
                profile: self.name.clone(),
                bindings,
            });
        }

        let mut chains: IndexMap<HumanoidChain, RigJointChain> = IndexMap::new();
        for chain in HumanoidChain::ALL {
            let resolved: RigJointChain = match self.chains.get(chain) {
                // An explicit chain lists the rig's own joints, extras included.
                Some(binding) => binding
                    .joints
                    .iter()
                    .filter_map(|name| index.find(name))
                    .collect(),
                // Otherwise the chain is whatever roles of it did resolve.
                None => chain
                    .joints()
                    .iter()
                    .filter_map(|role| joints.get(role).copied())
                    .collect(),
            };
            if resolved.presence() == ChainPresence::Populated {
                chains.insert(*chain, resolved);
            }
        }

        let root = match &self.root {
            RootSource::Pelvis => joints.get(&HumanoidJoint::Pelvis).copied(),
            RootSource::Joint(name) => index.find(name),
            RootSource::None => None,
        };

        let claimed: std::collections::HashSet<RigJointOrdinal> = joints
            .values()
            .copied()
            .chain(chains.values().flatten().copied())
            .chain(root)
            .collect();
        let unmapped = skeleton
            .joints
            .ordinals()
            .filter(|index| !claimed.contains(index))
            .collect();

        Ok(ResolvedRig {
            profile: self.name.clone(),
            joints,
            chains,
            root,
            missing,
            unmapped,
        })
    }

    /// Whether a skeleton looks like this rig, by its marker joints.
    ///
    /// Detection is a convenience for tooling. It is never required: an
    /// explicit profile always works, and a rig with no markers simply never
    /// auto-detects.
    pub fn matches(&self, skeleton: &Skeleton) -> bool {
        if self.markers.is_empty() {
            return false;
        }
        let index = JointIndex::new(skeleton);
        self.markers.iter().all(|name| index.find(name).is_some())
    }
}

/// Both rigs bound to their skeletons, plus the policy joining them.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedProfile {
    pub name: RetargetProfileName,
    pub source: ResolvedRig,
    pub target: ResolvedRig,
    pub settings: RetargetSettings,
}

impl RetargetProfile {
    pub fn resolve(
        &self,
        source_skeleton: &Skeleton,
        target_skeleton: &Skeleton,
    ) -> Result<ResolvedProfile, RetargetError> {
        let source = self.source.resolve(source_skeleton)?;
        let target = self.target.resolve(target_skeleton)?;

        if self.settings.strict == RetargetStrictness::Strict {
            let orphaned: Vec<HumanoidJoint> = source
                .joints
                .keys()
                .filter(|role| !target.has(**role))
                .copied()
                .collect();
            if !orphaned.is_empty() {
                return Err(RetargetError::StrictTargetRolesMissing { roles: orphaned });
            }
        }

        Ok(ResolvedProfile {
            name: self.name.clone(),
            source,
            target,
            settings: self.settings.clone(),
        })
    }
}

impl ResolvedProfile {
    /// Roles both rigs resolved, which is what actually gets retargeted.
    pub fn shared_roles(&self) -> Vec<HumanoidJoint> {
        HumanoidJoint::ALL
            .iter()
            .copied()
            .filter(|role| self.source.has(*role) && self.target.has(*role))
            .collect()
    }
}
