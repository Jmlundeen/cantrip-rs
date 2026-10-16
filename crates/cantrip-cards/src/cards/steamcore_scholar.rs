use cantrip_api::*;
use cantrip_mechanics::{choose, discard_chosen, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, draw two cards. Then discard two cards unless you discard an instant or sorcery card
/// or a creature card with flying." The player decides as the discard is made (CR 608.2d) between the one qualifying
/// card and two cards of any kind; with no qualifying card in hand the choice is not offered. Two cards are chosen
/// freely, so a qualifying card may be one of them (Scryfall ruling, 2024-02-02).
fn enters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let qualifying: Vec<ObjectId> = hand
        .iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            chars.is(CardType::Instant)
                || chars.is(CardType::Sorcery)
                || (chars.is(CardType::Creature) && chars.has(Keyword::Flying))
        })
        .collect();
    if !qualifying.is_empty() {
        let card = g.obj(item.object).card;
        if choose::one_of(g, you, item.object, vec![Label::Card(card, 0), Label::Card(card, 1)])? == 0 {
            let chosen = choose::cards_from(g, you, item.object, Purpose::Discard, hand, qualifying, 1, false)?;
            g.discard_together(&chosen);
            return Ok(());
        }
    }
    discard_chosen(g, you, item.object, 2)?;
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
