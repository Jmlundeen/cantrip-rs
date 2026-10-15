use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// 2 damage to any target; 4 instead if it was kicked.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, if item.kicked > 0 { 4 } else { 2 });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: mana(4, 0, 0, 0, 0, 0), multi: false, waterbend: false }),
    spell: Some(SpellDef { targets: &[targets::ANY], effect }),
    ..oracle::BURST_LIGHTNING
};
