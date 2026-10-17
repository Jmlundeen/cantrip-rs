use cantrip_api::*;
use cantrip_mechanics::{damage_target, delirium, targets};
use cantrip_oracle as oracle;

/// Delirium (an ability word, CR 207.2c): four or more card types among the cards in the controller's graveyard, counted as it
/// resolves.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let amount = if delirium(g, item.controller) { 6 } else { 2 };
    damage_target(g, item, 0, amount);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::UNHOLY_HEAT
};
