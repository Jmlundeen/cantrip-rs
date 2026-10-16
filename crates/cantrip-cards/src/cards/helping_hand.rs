use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return target creature card ... from your graveyard to the battlefield tapped." If the target is gone, the spell
/// is countered on resolution (CR 608.2b) and never gets here; the one target is legal, so `object` has it (iterating the Option keeps a dead else arm out).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for card in targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD], effect),
    ..oracle::HELPING_HAND
};
