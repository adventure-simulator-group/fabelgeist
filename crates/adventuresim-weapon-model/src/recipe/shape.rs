//! Precise manufacturing parameters owned by the canonical weapon kernel.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Shape {
    #[serde(rename = "bentBar")]
    BentBar(BentBarParameters),
    #[serde(rename = "spatialTube")]
    SpatialTube(SpatialTubeParameters),
    #[serde(rename = "loftedBlade")]
    LoftedBlade(LoftedBladeParameters),
    #[serde(rename = "shaft")]
    Shaft(Shaft),
    #[serde(rename = "firearm")]
    Firearm(FirearmParameters),
    #[serde(rename = "leadBall")]
    LeadBall(LeadBallParameters),
    #[serde(rename = "ballPouch")]
    BallPouch(BallPouchParameters),
    #[serde(rename = "crossbow")]
    Crossbow(CrossbowParameters),
    #[serde(rename = "crossbowBolt")]
    CrossbowBolt(CrossbowBoltParameters),
    #[serde(rename = "boltQuiver")]
    BoltQuiver(BoltQuiverParameters),
    #[serde(rename = "archeryBow")]
    ArcheryBow(ArcheryBowParameters),
    #[serde(rename = "arrow")]
    Arrow(ArrowParameters),
    #[serde(rename = "arrowQuiver")]
    ArrowQuiver(ArrowQuiverParameters),
    #[serde(rename = "blade")]
    Blade(BladeParameters),
    #[serde(rename = "sectionBlade")]
    SectionBlade(SectionBladeParameters),
    #[serde(rename = "diamondBlade")]
    DiamondBlade(DiamondBladeParameters),
    #[serde(rename = "axe")]
    Axe(AxeParameters),
    #[serde(rename = "spear")]
    Spear(SpearParameters),
    #[serde(rename = "guard")]
    Guard(GuardParameters),
    #[serde(rename = "guardAssembly")]
    GuardAssembly(GuardAssemblyParameters),
    #[serde(rename = "knuckleBow")]
    KnuckleBow(KnuckleBowParameters),
    #[serde(rename = "ringGuard")]
    RingGuard(RingGuardParameters),
    #[serde(rename = "tube")]
    Tube(TubeParameters),
    #[serde(rename = "figureEight")]
    FigureEight(FigureEightParameters),
    #[serde(rename = "fork")]
    Fork(ForkParameters),
    #[serde(rename = "partisan")]
    Partisan(PartisanParameters),
    #[serde(rename = "glaive")]
    Glaive(GlaiveParameters),
    #[serde(rename = "facetedBeak")]
    FacetedBeak(FacetedBeakParameters),
    #[serde(rename = "bill")]
    Bill(BillParameters),
    #[serde(rename = "box")]
    Box(BoxParameters),
    #[serde(rename = "pick")]
    Pick(PickParameters),
    #[serde(rename = "beak")]
    Beak(BeakParameters),
    #[serde(rename = "hammer")]
    Hammer(HammerParameters),
    #[serde(rename = "socket")]
    Socket(SocketParameters),
    #[serde(rename = "pommel")]
    Pommel(PommelParameters),
    #[serde(rename = "collar")]
    Collar(CollarParameters),
    #[serde(rename = "sleeve")]
    Sleeve(SleeveParameters),
    #[serde(rename = "mace")]
    Mace(MaceParameters),
    #[serde(rename = "grip")]
    Grip(GripParameters),
    #[serde(rename = "ovalGrip")]
    OvalGrip(OvalGripParameters),
    #[serde(rename = "slabGrip")]
    SlabGrip(SlabGripParameters),
    #[serde(rename = "roundShield")]
    RoundShield(RoundShieldParameters),
    #[serde(rename = "shapedShield")]
    ShapedShield(ShapedShieldParameters),
}
