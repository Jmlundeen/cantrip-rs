use cantrip_api::*;
use cantrip_mechanics::{library, sneak};
use cantrip_oracle as oracle;

/// "Creature cards" (CR 205.2a), read in the library where they are.
fn creature_card(q: &dyn Query, card: ObjectId) -> bool {
    q.chars(card).is(CardType::Creature)
}

/// "Look at the top eight cards of your library. Put up to two creature cards with total mana value 6 or less from among
/// them onto the battlefield and the rest on the bottom of your library in a random order." The two are chosen together, their
/// mana values read in the library (an {X} there is 0, CR 202.3e) adding up to 6 or less, which the question itself enforces;
/// they enter together under your control (CR 110.2a), and the other cards go to the bottom at random (CR 401.4). A
/// library of fewer than eight shows what it has (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 8, 2, "creature", creature_card, Some(6), library::Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[sneak::way(mana("{3}{G}"), "Sneak {3}{G}")],
    ..oracle::MICHELANGELOS_TECHNIQUE
};
