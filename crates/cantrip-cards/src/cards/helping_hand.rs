use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature card with mana value 3 or less from your graveyard."
fn small_creature_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 3)
}

/// "Return target creature card ... from your graveyard to the battlefield tapped." If the target is gone, the spell
/// is countered on resolution (CR 608.2b) and never gets here; the one target is legal, so `legal_objects` has it.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for card in targets::legal_objects(g, item) {
        g.put_onto_battlefield(card, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(small_creature_in_your_graveyard)], effect),
    ..oracle::HELPING_HAND
};
