use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// Delirium (CR 702.131): four or more card types among the cards in the controller's graveyard, counted as it
/// resolves.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut types = TypeSet::NONE;
    for &id in g.state().zone(Zone::graveyard(item.controller)) {
        for t in g.chars(id).types.iter() {
            types = types.with(t);
        }
    }
    let amount = if types.iter().count() >= 4 { 6 } else { 2 };
    damage_target(g, item, 0, amount);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::UNHOLY_HEAT
};
