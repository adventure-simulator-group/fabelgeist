//! Per-construction dimension and allocation bounds.
use super::*;
mod archery;
mod blades;
mod crossbow;
mod firearm;
mod furniture;
mod guards;
mod shields;
mod union;
pub(super) fn check(shape: &Shape) -> Checked {
    match shape {
        Shape::BentBar(p) => union::bent_bar(p),
        Shape::SpatialTube(p) => union::spatial_tube(p),
        Shape::LoftedBlade(p) => union::lofted_blade(p),
        Shape::Shaft(p) => union::shaft(p),
        Shape::Firearm(p) => firearm::firearm(p),
        Shape::LeadBall(p) => firearm::lead_ball(p),
        Shape::BallPouch(p) => firearm::ball_pouch(p),
        Shape::Crossbow(p) => crossbow::crossbow(p),
        Shape::CrossbowBolt(p) => crossbow::crossbow_bolt(p),
        Shape::BoltQuiver(p) => crossbow::bolt_quiver(p),
        Shape::ArcheryBow(p) => archery::archery_bow(p),
        Shape::Arrow(p) => archery::arrow(p),
        Shape::ArrowQuiver(p) => archery::arrow_quiver(p),
        Shape::Blade(p) => blades::blade(p),
        Shape::SectionBlade(p) => blades::section_blade(p),
        Shape::DiamondBlade(p) => blades::diamond_blade(p),
        Shape::Axe(p) => blades::axe(p),
        Shape::Spear(p) => blades::spear(p),
        Shape::Guard(p) => guards::guard(p),
        Shape::GuardAssembly(p) => guards::guard_assembly(p),
        Shape::KnuckleBow(p) => guards::knuckle_bow(p),
        Shape::RingGuard(p) => guards::ring_guard(p),
        Shape::Tube(p) => guards::tube(p),
        Shape::FigureEight(p) => guards::figure_eight(p),
        Shape::Fork(p) => blades::fork(p),
        Shape::Partisan(p) => blades::partisan(p),
        Shape::Glaive(p) => blades::glaive(p),
        Shape::FacetedBeak(p) => blades::faceted_beak(p),
        Shape::Bill(p) => blades::bill(p),
        Shape::Box(p) => furniture::cuboid(p),
        Shape::Pick(p) => blades::pick(p),
        Shape::Beak(p) => blades::beak(p),
        Shape::Hammer(p) => blades::hammer(p),
        Shape::Socket(p) => furniture::socket(p),
        Shape::Pommel(p) => furniture::pommel(p),
        Shape::Collar(p) => furniture::collar(p),
        Shape::Sleeve(p) => furniture::sleeve(p),
        Shape::Mace(p) => furniture::mace(p),
        Shape::Grip(p) => furniture::grip(p),
        Shape::OvalGrip(p) => furniture::oval_grip(p),
        Shape::SlabGrip(p) => furniture::slab_grip(p),
        Shape::RoundShield(p) => shields::round_shield(p),
        Shape::ShapedShield(p) => shields::shaped_shield(p),
    }
}
