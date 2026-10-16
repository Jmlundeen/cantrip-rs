use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Destroy target creature if it has mana value 2 or less. Revolt: 4 or less instead, if a permanent left the
/// battlefield under your control this turn (CR 207.2c). The mana value is the creature's as it resolves; nothing
/// happens at all if it is no longer a legal target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let value = g.mana_value(creature);
        let revolt = cantrip_mechanics::history::permanent_left_battlefield_this_turn(g, item.controller);
        if value <= if revolt { 4 } else { 2 } {
            g.destroy(creature);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::FATAL_PUSH
};
