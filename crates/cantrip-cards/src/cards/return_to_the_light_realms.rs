use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return all nonland permanent cards from your graveyard to the battlefield" (CR 400.3, 110.2a): every card that is
/// both a permanent card and not a land, in the graveyard as the spell resolves, entering together under their owner's
/// control (a card in a graveyard is its owner's, CR 108.4a).
fn return_all(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let cards: Vec<(ObjectId, PlayerId)> = g
        .state()
        .zone(Zone::graveyard(you))
        .iter()
        .copied()
        .filter(|&card| targets::is_permanent_card(&g.chars(card)) && !g.chars(card).is(CardType::Land))
        .map(|card| (card, g.obj(card).owner))
        .collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], return_all),
    ..oracle::RETURN_TO_THE_LIGHT_REALMS
};
