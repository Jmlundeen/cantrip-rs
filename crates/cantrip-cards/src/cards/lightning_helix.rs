use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Lightning Helix deals 3 damage to any target and you gain 3 life." The life is gained whether or not the target is
/// still legal: only a spell whose every target is illegal does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    g.gain_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[targets::ANY], effect }), ..oracle::LIGHTNING_HELIX };
