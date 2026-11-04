use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "{8}: Until end of turn, this creature gets +1/+1 for each permanent card in your graveyard.": counted once, as the ability
/// resolves, and fixed from then on (CR 608.2h, 611.2c); a permanent card is an artifact, battle, creature, enchantment, land or
/// planeswalker card (CR 110.4a), the controller's graveyard alone. Nothing if it has left the battlefield (CR 400.7).
fn grow_per_permanent_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::permanent_cards_in_graveyard(g, item.controller) as i32;
    pump(g, item.object, item.object, cards, cards);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{8}"),
        grow_per_permanent_card,
        "{8}: Until end of turn, this creature gets +1/+1 for each permanent card in your graveyard.",
    )],
    ..oracle::GRAN_PULSE_OCHU
};
