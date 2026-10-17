use cantrip_api::*;
use cantrip_mechanics::{choose_discard_two_unless_discarding, harmonize};
use cantrip_oracle as oracle;

/// "Draw three cards. Then discard two cards unless you discard a creature card." The player decides as the discard is made
/// (CR 608.2d) between one creature card and two cards of any kind; with no creature card in hand the choice is not offered. Two
/// cards are chosen freely, so a creature card may be one of them.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 3);
    let chosen = choose_discard_two_unless_discarding(g, you, item.object, |chars| chars.is(CardType::Creature))?;
    g.discard_together(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[harmonize::way(mana("{4}{U}"), "Harmonize {4}{U}")],
    labels: &["Discard a creature card", "Discard two cards"],
    ..oracle::WINTERNIGHT_STORIES
};
