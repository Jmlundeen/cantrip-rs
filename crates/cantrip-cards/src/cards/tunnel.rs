use cantrip_api::*;
use cantrip_mechanics::{destroy_target_no_regen, targets};
use cantrip_oracle as oracle;

/// Target a Wall, including a permanent whose current creature types include Wall (CR 115.1, 612.2).
fn wall(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Wall])
}

/// Destroy the target without allowing regeneration (CR 701.8a, 701.19c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_no_regen(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(wall)], effect),
    ..oracle::TUNNEL
};
