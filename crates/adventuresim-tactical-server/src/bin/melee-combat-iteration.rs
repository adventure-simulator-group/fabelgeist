//! Accelerated deterministic melee iteration command.

mod melee_combat_iteration;

fn main() -> melee_combat_iteration::IterationResult<()> {
    melee_combat_iteration::run()
}
