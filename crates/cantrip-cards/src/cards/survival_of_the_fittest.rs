use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_oracle as oracle;

/// "Discard a creature card": a creature card in the activator's hand.
fn creature_card(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    q.chars(card).is(CardType::Creature)
}

/// Search your library for a creature card, reveal it, put it into your hand, then shuffle. The search may come up
/// empty; the shuffle follows either way.
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Creature))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{G}, Discard a creature card: Search your library for a creature card, reveal that card, put it into your hand, then shuffle.",
        cost: Cost { mana: mana(0, 0, 0, 0, 0, 1), discard: Some((1, creature_card)), ..Cost::NONE },
        effect: tutor,
        ..ActivatedDef::BLANK
    })],
    ..oracle::SURVIVAL_OF_THE_FITTEST
};
