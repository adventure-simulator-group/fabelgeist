//! Manufacturing and numerical geometry failures shared by model consumers.
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum ConstructionError {
    #[error("stock stations need positive sections and increasing axial positions")]
    StockStations,
    #[error("receiving cut leaves a fragment below the manufacturing minimum")]
    CutManufacturingMinimum,
    #[error("mounted blade must match the guard mortise section, engagement and pose")]
    GuardMortise,
    #[error("plate profile permits zero thickness only at isolated authored boundary apices")]
    PlateBoundaryApices,
    #[error("missing guard anchor node")]
    MissingGuardAnchor,
    #[error("missing plate node {node}")]
    MissingPlateNode { node: String },
    #[error("invalid weapon recipe: {0}")]
    Recipe(#[source] crate::recipe::RecipeError),
    #[error("construction quantity must be finite")]
    Quantity(#[from] crate::recipe::NonFiniteQuantity),
    #[error("attachment overlap cannot be negative")]
    AttachmentOverlapCannotNegative,
    #[error("binding references missing guard node")]
    BindingReferencesMissingGuardNode,
    #[error("blade sampling exceeds its surface-error budget")]
    BladeSamplingExceedsSurfaceErrorBudget,
    #[error("blade section contains a nonfinite vertex")]
    BladeSectionContainsNonfiniteVertex,
    #[error("blade section has no area")]
    BladeSectionNoArea,
    #[error("bow core must retain positive depth between laminations")]
    BowCoreRetainPositiveDepthBetweenLaminations,
    #[error("clearance envelope needs three supporting planes")]
    ClearanceEnvelopeNeedsThreeSupportingPlanes,
    #[error("clearance envelope needs two axial sections")]
    ClearanceEnvelopeNeedsTwoAxialSections,
    #[error("closed member needs whole-turn section twist")]
    ClosedMemberNeedsWholeTurnSectionTwist,
    #[error("component has no directional working end")]
    ComponentNoDirectionalWorkingEnd,
    #[error("component-end needs anchor")]
    ComponentEndNeedsAnchor,
    #[error("composite bow needs backing thickness")]
    CompositeBowNeedsBackingThickness,
    #[error("composite bow needs horn thickness")]
    CompositeBowNeedsHornThickness,
    #[error("composite pommel needs baseConstruction")]
    CompositePommelNeedsBaseConstruction,
    #[error("composite pommel needs ornaments")]
    CompositePommelNeedsOrnaments,
    #[error("composite pommel needs sockets")]
    CompositePommelNeedsSockets,
    #[error("composite prod needs horn thickness")]
    CompositeProdNeedsHornThickness,
    #[error("composite prod needs sinew thickness")]
    CompositeProdNeedsSinewThickness,
    #[error("crenellated socket needs a wall or receiver")]
    CrenellatedSocketNeedsWallReceiver,
    #[error("crossbow core must retain positive depth")]
    CrossbowCoreRetainPositiveDepth,
    #[error("double barrel needs secondary length")]
    DoubleBarrelNeedsSecondaryLength,
    #[error("faceted socket requires explicit shared-section attachment")]
    FacetedSocketRequiresExplicitSharedSectionAttachment,
    #[error("faceted solid requires stations and at least three sides")]
    FacetedSolidRequiresStationsThreeSides,
    #[error("frame needs owner and name")]
    FrameNeedsOwnerName,
    #[error("fuller strips cannot be resolved within the surface-error budget")]
    FullerSurfaceBudget,
    #[error("grip cover inset collapses its core section")]
    GripCoverInsetCollapsesCoreSection,
    #[error("groove transition exceeds its bounded cutoff refinement")]
    GrooveTransitionExceedsBoundedCutoffRefinement,
    #[error("guard assembly needs anchorNode")]
    GuardAssemblyNeedsAnchorNode,
    #[error("guard assembly needs nodes")]
    GuardAssemblyNeedsNodes,
    #[error("guard node bindings contain a cycle")]
    GuardNodeBindingsContainCycle,
    #[error("guard plate loops need equal station counts")]
    GuardPlateLoopsNeedEqualStationCounts,
    #[error("hand control point must remain within the modeled grip")]
    HandControlPointRemainModeledGrip,
    #[error("heel-center requires a generic blade")]
    HeelCenterRequiresGenericBlade,
    #[error("lathed pommel needs profile")]
    LathedPommelNeedsProfile,
    #[error("mace core face reaches outside the flange outline")]
    MaceFlangeOutline,
    #[error("mace core sides must be a multiple of its flange count")]
    MaceCoreSidesMultipleFlangeCount,
    #[error("mace core stations must have strictly increasing heights")]
    MaceCoreStationsHaveStrictlyIncreasingHeights,
    #[error("mace flange thickness exceeds its receiving core face")]
    MaceFlangeThicknessExceedsReceivingCoreFace,
    #[error("member centerline intersects itself")]
    MemberCenterlineIntersectsItself,
    #[error("member centerline reverses direction")]
    MemberCenterlineReversesDirection,
    #[error("member has repeated adjacent stations")]
    MemberRepeatedAdjacentStations,
    #[error("member needs at least two named nodes")]
    MemberNeedsTwoNamedNodes,
    #[error("member needs at least two stations")]
    MemberNeedsTwoStations,
    #[error("missing interpolation node")]
    MissingInterpolationNode,
    #[error("missing ornament socket")]
    MissingOrnamentSocket,
    #[error("missing stretch source frame")]
    MissingStretchSourceFrame,
    #[error("missing stretch target frame")]
    MissingStretchTargetFrame,
    #[error("notches leave no socket base")]
    NotchesLeaveNoSocketBase,
    #[error("opposed working end names a missing component")]
    OpposedWorkingEndNamesMissingComponent,
    #[error("ornament requires complete vertices and triangles")]
    OrnamentRequiresCompleteVerticesTriangles,
    #[error("ornament vertex index out of range")]
    OrnamentVertexIndexOutRange,
    #[error("outline collapsed during triangulation")]
    OutlineCollapsedDuringTriangulation,
    #[error("outline edges intersect or touch")]
    OutlineEdgesIntersectTouch,
    #[error("outline has a degenerate final triangle")]
    OutlineDegenerateFinalTriangle,
    #[error("outline has no extent")]
    OutlineNoExtent,
    #[error("outline must be simple and nondegenerate")]
    OutlineSimpleNondegenerate,
    #[error("outline needs at least three finite points")]
    OutlineNeedsThreeFinitePoints,
    #[error("plate cell boundary has an unresolved junction")]
    PlateCellBoundaryUnresolvedJunction,
    #[error("plate cell boundary is degenerate")]
    PlateCellBoundaryDegenerate,
    #[error("plate cell boundary is open")]
    PlateCellBoundaryOpen,
    #[error("plate cut leaves a sub-resolution feature")]
    PlateCutResolution,
    #[error("plate needs three nodes")]
    PlateNeedsThreeNodes,
    #[error("plate outline exceeds its bounded sampling budget")]
    PlateOutlineExceedsBoundedSamplingBudget,
    #[error("plate profile leaves a zero-thickness surface")]
    PlateProfileLeavesZeroThicknessSurface,
    #[error("plate surface exceeds its bounded refinement budget")]
    PlateSurfaceExceedsBoundedRefinementBudget,
    #[error("point needs a positive monotonically tapering body section")]
    PointTaper,
    #[error("profile terminal needs stations")]
    ProfileTerminalNeedsStations,
    #[error("profile terminal requires its authored stations")]
    ProfileTerminalRequiresAuthoredStations,
    #[error("quillon crosses the terminal joint plane")]
    QuillonCrossesTerminalJointPlane,
    #[error("quiver wall must leave an open interior")]
    QuiverWallLeaveOpenInterior,
    #[error("radial profile requires at least two stations")]
    RadialProfileRequiresTwoStations,
    #[error("radial solid requires stations and a positive section")]
    RadialSolidRequiresStationsPositiveSection,
    #[error("receiving cut cannot retain a closed float32 boundary")]
    ReceivingCutCannotRetainClosedFloat32Boundary,
    #[error("receiving cut has an unresolved float32 surface")]
    ReceivingCutUnresolvedFloat32Surface,
    #[error("receiving plane must cut a finite convex section")]
    ReceivingPlaneCutFiniteConvexSection,
    #[error("receiving socket must align with its shaft")]
    ReceivingSocketAlignWithShaft,
    #[error("receiving socket must seat over an unwrapped shaft tenon")]
    ReceivingSocketSeatOverUnwrappedShaftTenon,
    #[error("resolved assembly exceeds the supported world extent")]
    ResolvedAssemblyExceedsSupportedWorldExtent,
    #[error("section scales must match member stations")]
    SectionScalesMatchMemberStations,
    #[error("shaft-mounted axial construction must seat concentrically")]
    ShaftMountedAxialConstructionSeatConcentrically,
    #[error("shaft-top mount requires shaft")]
    ShaftTopMountRequiresShaft,
    #[error("shell requires matching closed section rings")]
    ShellRequiresMatchingClosedSectionRings,
    #[error("shield fitting layout has no interior clearance")]
    ShieldFittingLayoutNoInteriorClearance,
    #[error("shield fitting lies outside panel surface")]
    ShieldFittingLiesOutsidePanelSurface,
    #[error("side fitting requires shaft")]
    SideFittingRequiresShaft,
    #[error("sleeve bore cannot fit shaft")]
    SleeveBoreCannotFitShaft,
    #[error("smooth profile needs finite increasing stations")]
    SmoothProfileNeedsFiniteIncreasingStations,
    #[error("socket bore cannot clear the complete receiving shaft")]
    SocketShaftClearance,
    #[error("socket bore cannot fit shaft")]
    SocketBoreCannotFitShaft,
    #[error("socket needs a profile")]
    SocketNeedsProfile,
    #[error("socket placement does not match its declared shaft insertion")]
    SocketPlacementDoesNotMatchDeclaredShaftInsertion,
    #[error("socket wall leaves no bore")]
    SocketWallLeavesNoBore,
    #[error("socketed blade needs socket dimensions")]
    SocketedBladeNeedsSocketDimensions,
    #[error("socketed blade needs stations")]
    SocketedBladeNeedsStations,
    #[error("socketed spear uses shaft-top receiving mount")]
    SocketedSpearUsesShaftTopReceivingMount,
    #[error("solid exceeds its bounded construction budget")]
    ConstructionBudget,
    #[error("spatial member needs points")]
    SpatialMemberNeedsPoints,
    #[error("stretch target must lie above source")]
    StretchTargetLieAboveSource,
    #[error("stretchBetween requires knuckle bow")]
    StretchBetweenRequiresKnuckleBow,
    #[error("stretched start and end must meet their declared frames")]
    StretchedStartEndMeetDeclaredFrames,
    #[error("sweep needs a positive section and at least two points")]
    SweepNeedsPositiveSectionTwoPoints,
    #[error("terminal base must cover its receiving quillon section")]
    TerminalQuillonCoverage,
    #[error("tube needs points")]
    TubeNeedsPoints,
    #[error("unresolved blade surface normal")]
    UnresolvedBladeSurfaceNormal,
    #[error("working-end facing dependencies contain a cycle")]
    WorkingEndFacingDependenciesContainCycle,
    #[error("{component}: missing frame {frame}")]
    MissingFrame { component: String, frame: String },
    #[error("missing bound node frame {frame}")]
    MissingBoundFrame { frame: String },
    #[error("missing member node {node}")]
    MissingMemberNode { node: String },
    #[error("declared contact lies outside parent axial geometry {parent}")]
    ContactOutsideParent { parent: String },
    #[error("{component} attachment lies outside parent axial geometry {parent}")]
    AttachmentOutsideParent { component: String, parent: String },
    #[error("{component} attachment lies outside parent footprint {parent}")]
    AttachmentOutsideFootprint { component: String, parent: String },
    #[error("fuller tail cannot be simplified within the normal-error budget: {0:?}")]
    FullerNormalBudget(NormalBudgetFailure),
}

/// The failed numerical tolerance check, retained for geometry diagnostics.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalBudgetFailure {
    pub interval: [f64; 2],
    pub side: usize,
    pub parameter: [f64; 2],
    pub analytic_to_flat: f64,
    pub mesh_to_analytic: f64,
    pub base_error: f64,
    pub tolerance: f64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn failed_quantity_remains_inspectable_through_generation_error() {
        let cause = crate::recipe::Metres::new(f64::INFINITY).unwrap_err();
        let error = crate::GenerateError::Construction(super::ConstructionError::Quantity(cause));
        let construction = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<super::ConstructionError>()
            .unwrap();
        assert!(matches!(
            construction,
            super::ConstructionError::Quantity(_)
        ));
        assert!(
            std::error::Error::source(construction)
                .unwrap()
                .is::<crate::recipe::NonFiniteQuantity>()
        );
    }
}
