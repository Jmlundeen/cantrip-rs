use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Lightning Helix deals 3 damage to any target and you gain 3 life." With its only target illegal the spell does not
/// resolve at all (CR 608.2b), so `damage_target` is only ever reached with the target legal; the check stays for the
/// effect's own safety.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    g.gain_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::ANY], effect), ..oracle::LIGHTNING_HELIX };
