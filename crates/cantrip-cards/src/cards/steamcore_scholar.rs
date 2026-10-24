use cantrip_api::*;
use cantrip_mechanics::{discard_two_unless_discarding, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, draw two cards. Then discard two cards unless you discard an instant or sorcery card
/// or a creature card with flying." The player decides as the discard is made (CR 608.2d) between the one qualifying
/// card and two cards of any kind; with no qualifying card in hand the choice is not offered. Two cards are chosen
/// freely, so a qualifying card may be one of them (Scryfall ruling, 2024-02-02).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    discard_two_unless_discarding(g, &[you], item.object, |chars| {
        chars.is(CardType::Instant) || chars.is(CardType::Sorcery) || (chars.is(CardType::Creature) && chars.has(Keyword::Flying))
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        enters,
        "When this creature enters, draw two cards. Then discard two cards unless you discard an instant or sorcery card or a creature card with flying.",
    )],
    labels: &["Discard an instant or sorcery card or a creature card with flying", "Discard two cards"],
    ..oracle::STEAMCORE_SCHOLAR
};
