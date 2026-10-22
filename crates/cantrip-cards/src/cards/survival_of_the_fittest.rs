use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Discard a creature card": a creature card in the activator's hand.
fn creature_card(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    q.chars(card).is(CardType::Creature)
}

/// Search your library for a creature card, reveal it, put it into your hand, then shuffle. The search may come up
/// empty; the shuffle follows either way.
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::CREATURE_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{G}").and(discard(1, creature_card)), tutor, "{G}, Discard a creature card: Search your library for a creature card, reveal that card, put it into your hand, then shuffle.")],
    ..oracle::SURVIVAL_OF_THE_FITTEST
};
