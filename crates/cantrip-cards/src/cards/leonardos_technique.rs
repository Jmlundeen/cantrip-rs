use cantrip_api::*;
use cantrip_mechanics::{sneak, targets};
use cantrip_oracle as oracle;

/// "Return one or two target creature cards each with mana value 3 or less from your graveyard to the battlefield." The
/// targets still legal (CR 608.2b) enter together under your control (CR 101.4, 400.7).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId)> =
        targets::legal_objects(g, item).into_iter().map(|card| (card, item.controller)).collect();
    g.put_all_onto_battlefield(&cards)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD.between(1, 2)], effect),
    ways: &[sneak::way(mana("{1}{W}"), "Sneak {1}{W}")],
    ..oracle::LEONARDOS_TECHNIQUE
};
