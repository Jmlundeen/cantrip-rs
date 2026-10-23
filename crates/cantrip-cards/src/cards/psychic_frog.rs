use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on this creature.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_OR_PLANESWALKER, |g, item| effects::draw(g, item, 1), "Whenever this creature deals combat damage to a player or planeswalker, draw a card."),
        ability(discard(1, filters::any), grow, "Discard a card: Put a +1/+1 counter on this creature."),
        ability(exile_from_graveyard(3), effects::this_gains_flying, "Exile three cards from your graveyard: This creature gains flying until end of turn."),
    ],
    ..oracle::PSYCHIC_FROG
};
