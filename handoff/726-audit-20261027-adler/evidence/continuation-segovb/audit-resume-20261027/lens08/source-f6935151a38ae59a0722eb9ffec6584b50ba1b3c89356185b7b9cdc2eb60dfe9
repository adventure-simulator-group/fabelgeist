use super::*;

#[derive(Clone, Debug, Serialize)]
pub struct StructuralNode {
    pub id: StructuralNodeId,
    pub owner: GeometryOwnerId,
    pub kind: StructuralNodeKind,
    pub position: crate::spatial_geometry::Position<Architectural>,
    pub supported_by: Vec<StructuralNodeId>,
    pub grounded: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("structural node {node:?}: {cause}")]
pub struct StructuralNodeError {
    pub node: StructuralNodeId,
    #[source]
    pub cause: crate::spatial_geometry::GeometryError,
}
impl StructuralNode {
    /// Admit a node emitted by an architectural set-out kernel, preserving its
    /// identity if the computed position is nonfinite.
    pub fn from_metres(
        id: StructuralNodeId,
        owner: GeometryOwnerId,
        kind: StructuralNodeKind,
        position: Vec3,
        supported_by: Vec<StructuralNodeId>,
        grounded: bool,
    ) -> Result<Self, StructuralNodeError> {
        let position = crate::spatial_geometry::Position::from_metres(position)
            .map_err(|cause| StructuralNodeError { node: id, cause })?;
        Ok(Self::new(id, owner, kind, position, supported_by, grounded))
    }
    pub fn new(
        id: StructuralNodeId,
        owner: GeometryOwnerId,
        kind: StructuralNodeKind,
        position: crate::spatial_geometry::Position<Architectural>,
        supported_by: Vec<StructuralNodeId>,
        grounded: bool,
    ) -> Self {
        Self {
            id,
            owner,
            kind,
            position,
            supported_by,
            grounded,
        }
    }
}
impl<'de> Deserialize<'de> for StructuralNode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct SerializedNode {
            id: StructuralNodeId,
            owner: GeometryOwnerId,
            kind: StructuralNodeKind,
            position: Vec3,
            supported_by: Vec<StructuralNodeId>,
            grounded: bool,
        }
        let value = SerializedNode::deserialize(deserializer)?;
        Self::from_metres(
            value.id,
            value.owner,
            value.kind,
            value.position,
            value.supported_by,
            value.grounded,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SupportInterface {
    pub id: ResolvedItemId,
    pub owner: GeometryOwnerId,
    pub node: StructuralNodeId,
    pub bounds: SpatialBounds<Architectural>,
}
impl SupportInterface {
    /// A bearing retains its physical owner and node; admitted bounds may have
    /// zero height. Positive contact is established by the bearing audit.
    pub fn new(
        id: ResolvedItemId,
        owner: GeometryOwnerId,
        node: StructuralNodeId,
        bounds: crate::SpatialBounds<crate::Architectural>,
    ) -> Self {
        Self {
            id,
            owner,
            node,
            bounds,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DrainageRoute {
    pub id: ResolvedItemId,
    pub owner: GeometryOwnerId,
    pub outlet_void: ResolvedItemId,
    pub inlet: Vec3,
    pub outlet: Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrainageCatchment {
    pub id: ResolvedItemId,
    pub owner: GeometryOwnerId,
    pub walk_solid: ResolvedItemId,
    pub toe_channel_solids: Vec<ResolvedItemId>,
    pub drainage_surface: ResolvedItemId,
    pub outlet_route: ResolvedItemId,
    pub centre: Vec3,
    /// Canonical local +X direction in plan.
    pub tangent: Vec2,
    /// Physical downhill direction in plan.
    pub outward: Vec2,
    pub length_metres: f32,
    pub width_metres: f32,
    pub inner_elevation_metres: f32,
    pub outer_elevation_metres: f32,
    /// Signed local-X coordinate of the exact scupper inlet at the channel end.
    pub outlet_along_metres: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RoofDrainageSample {
    /// Point sampled on the authoritative weather face.
    pub surface_point: Vec3,
    /// First physical contact with the receiving eave/valley channel.
    pub channel_inlet: Vec3,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoofDrainageNetwork {
    pub id: ResolvedItemId,
    pub owner: GeometryOwnerId,
    pub face: ResolvedItemId,
    pub catchment: ResolvedItemId,
    pub receiving_edge: ResolvedItemId,
    pub samples: Vec<RoofDrainageSample>,
    pub channel_floor: ResolvedItemId,
    pub channel_lips: [ResolvedItemId; 2],
    /// Physical perimeter collector segments connecting this catchment gutter
    /// to its shared outlet station.
    pub collector_solids: Vec<ResolvedItemId>,
    pub outlet_station: ResolvedItemId,
    pub outlet_void: ResolvedItemId,
    pub downspout: Option<ResolvedItemId>,
    pub channel_high: Vec3,
    pub channel_low: Vec3,
    pub discharge: Vec3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoofDrainageDisposition {
    BoundDownspout,
    FreeDripToParentRoof,
    FreeDripToGround,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RoofDrainageRecipient {
    GroundSplashApron,
    ParentRoofFace {
        roof: RoofAssemblyId,
        face: ResolvedItemId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoofDrainageOutletStation {
    pub id: ResolvedItemId,
    pub owner: GeometryOwnerId,
    pub disposition: RoofDrainageDisposition,
    pub member_networks: Vec<ResolvedItemId>,
    pub host_wall: Option<WallAssemblyId>,
    pub facade_contact: Option<Vec3>,
    pub outlet_void: ResolvedItemId,
    pub downspout: Option<ResolvedItemId>,
    pub recipient: RoofDrainageRecipient,
    pub recipient_surface: ResolvedItemId,
    pub discharge: Vec3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DefenderSample {
    pub owner: GeometryOwnerId,
    pub stance: Vec3,
    pub eye: Vec3,
    pub target: Vec3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct JunctionBond {
    pub id: ResolvedItemId,
    pub owners: [GeometryOwnerId; 2],
    pub bounds: SpatialBounds<Architectural>,
    pub minimum_interface_area_square_metres: f32,
    pub maximum_penetration_metres: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseKind {
    Machicolation,
    Breteche,
    Hoarding,
    Bartizan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseMaterial {
    Masonry,
    Timber,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefensePhase {
    PermanentMainWork,
    TemporaryCampaignWork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseDeployment {
    Permanent,
    SocketsOnly,
    Deployed,
}

/// Tactical reason for installing a projected defense. These labels keep
/// curated full-building fixtures from becoming an ahistorical catalogue of
/// unrelated devices merely because the resolver can construct them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseTarget {
    GateApproach,
    ThreatenedWallFoot,
    ThreatenedCorner,
    CampaignSiegeFront,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ProjectedDefensePath {
    Linear {
        start: Vec2,
        end: Vec2,
        outward: Direction,
    },
    Round {
        centre: Vec2,
        radius_metres: f32,
        outward: Direction,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ProjectedDefenseRay {
    pub owner: GeometryOwnerId,
    pub throat: ResolvedItemId,
    pub stance: Vec3,
    pub origin: Vec3,
    pub target: Vec3,
    pub range: ProjectedDefenseRange,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseRange {
    Near,
    Middle,
    Far,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ProjectedDefenseWorkingPoint {
    pub owner: GeometryOwnerId,
    pub aperture: ResolvedItemId,
    pub stance: Vec3,
    pub eye: Vec3,
    pub support_solid: ResolvedItemId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedDefenseHostTopology {
    LinearFace,
    CornerFaces,
    Buttress,
}

/// Exact source-wall identity replaced by the resolved host masonry.
///
/// The renderer suppresses these legacy wall cells and draws the resolved,
/// opening-aware replacement instead. This prevents a projected defense from
/// manufacturing an additive witness wall unrelated to the building model.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectedDefenseHostWallSource {
    pub storey_level: u16,
    pub wall_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectedDefenseAssembly {
    pub owner: GeometryOwnerId,
    /// Authoritative resolved masonry host. It is deliberately a distinct
    /// owner so junction bonds and void subtraction cannot be satisfied by
    /// projection-owned witness geometry.
    pub host_owner: GeometryOwnerId,
    pub host_wall_solids: Vec<ResolvedItemId>,
    pub host_buttress_solids: Vec<ResolvedItemId>,
    pub host_source_walls: Vec<ProjectedDefenseHostWallSource>,
    pub host_top_elevation_metres: f32,
    pub host_topology: ProjectedDefenseHostTopology,
    pub host_walk_solid: ResolvedItemId,
    pub host_portal_void: Option<ResolvedItemId>,
    pub host_bond: Option<ResolvedItemId>,
    pub beam_socket_voids: Vec<ResolvedItemId>,
    pub socket_joists: Vec<(ResolvedItemId, ResolvedItemId)>,
    pub kind: ProjectedDefenseKind,
    pub material: ProjectedDefenseMaterial,
    pub phase: ProjectedDefensePhase,
    pub deployment: ProjectedDefenseDeployment,
    pub tactical_target: ProjectedDefenseTarget,
    pub path: ProjectedDefensePath,
    pub floor_elevation_metres: f32,
    pub clear_width_metres: f32,
    pub clear_height_metres: f32,
    pub projection_metres: f32,
    pub breastwork_height_metres: f32,
    pub roofed: bool,
    pub floor_solids: Vec<ResolvedItemId>,
    pub throat_voids: Vec<ResolvedItemId>,
    pub access_portal: Option<ResolvedItemId>,
    pub access_landing: Option<ResolvedItemId>,
    pub firing_apertures: Vec<ResolvedItemId>,
    pub support_nodes: Vec<StructuralNodeId>,
    pub drain_route: Option<ResolvedItemId>,
    pub drainage_catchments: Vec<ResolvedItemId>,
    /// Roof or exposed coping catchments, distinct from the occupied floor.
    pub weather_catchments: Vec<ResolvedItemId>,
    pub weathering_solids: Vec<ResolvedItemId>,
    /// Physical enclosure walls/posts and wall plates carrying a roof. Empty
    /// for unroofed work; authoritative rather than proof-only geometry.
    pub roof_support_solids: Vec<ResolvedItemId>,
    /// Roof-bearing node whose parents are the independently supported inner
    /// and outer plate lines.
    pub roof_bearing_node: Option<StructuralNodeId>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResolvedGeometry {
    pub schema_version: u16,
    pub solids: Vec<ResolvedSolid>,
    pub surfaces: Vec<ResolvedSurface>,
    pub voids: Vec<ResolvedVoid>,
    pub structural_nodes: Vec<StructuralNode>,
    pub support_interfaces: Vec<SupportInterface>,
    pub drainage_routes: Vec<DrainageRoute>,
    pub drainage_catchments: Vec<DrainageCatchment>,
    pub roof_drainage_networks: Vec<RoofDrainageNetwork>,
    pub roof_drainage_outlets: Vec<RoofDrainageOutletStation>,
    pub defender_samples: Vec<DefenderSample>,
    pub junction_bonds: Vec<JunctionBond>,
    pub projected_defense_rays: Vec<ProjectedDefenseRay>,
    pub projected_defense_working_points: Vec<ProjectedDefenseWorkingPoint>,
}

impl ResolvedGeometry {
    pub fn solid_mut(
        &mut self,
        id: ResolvedItemId,
    ) -> Result<&mut ResolvedSolid, crate::GenerationError> {
        self.solids
            .iter_mut()
            .find(|solid| solid.id == id)
            .ok_or(crate::GenerationError::MissingSolid { solid: id })
    }
}

impl ResolvedGeometry {
    pub fn solid(&self, id: ResolvedItemId) -> Result<&ResolvedSolid, crate::GenerationError> {
        self.solids
            .iter()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingSolid { solid: id })
    }
    pub fn surface(&self, id: ResolvedItemId) -> Result<&ResolvedSurface, crate::GenerationError> {
        self.surfaces
            .iter()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingSurface { surface: id })
    }
    pub fn surface_mut(
        &mut self,
        id: ResolvedItemId,
    ) -> Result<&mut ResolvedSurface, crate::GenerationError> {
        self.surfaces
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingSurface { surface: id })
    }
    pub fn void(&self, id: ResolvedItemId) -> Result<&ResolvedVoid, crate::GenerationError> {
        self.voids
            .iter()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingVoid { void: id })
    }
    pub fn void_mut(
        &mut self,
        id: ResolvedItemId,
    ) -> Result<&mut ResolvedVoid, crate::GenerationError> {
        self.voids
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingVoid { void: id })
    }
    pub fn node(&self, id: StructuralNodeId) -> Result<&StructuralNode, crate::GenerationError> {
        self.structural_nodes
            .iter()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingNode { node: id })
    }
    pub fn node_mut(
        &mut self,
        id: StructuralNodeId,
    ) -> Result<&mut StructuralNode, crate::GenerationError> {
        self.structural_nodes
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingNode { node: id })
    }
    pub fn interface(
        &self,
        id: ResolvedItemId,
    ) -> Result<&SupportInterface, crate::GenerationError> {
        self.support_interfaces
            .iter()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingInterface { interface: id })
    }
    pub fn interface_mut(
        &mut self,
        id: ResolvedItemId,
    ) -> Result<&mut SupportInterface, crate::GenerationError> {
        self.support_interfaces
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(crate::GenerationError::MissingInterface { interface: id })
    }
}
