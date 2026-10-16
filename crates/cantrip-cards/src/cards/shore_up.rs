use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target creature you control gets +1/+1 and gains hexproof until end of turn. Untap it. All three do nothing if the
/// target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        cantrip_mechanics::pump(g, item.object, creature, 1, 1);
        cantrip_mechanics::effects::gain_hexproof_until_end_of_turn(g, item.object, creature);
        g.untap(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::SHORE_UP
};
