use cantrip_api::*;
use cantrip_mechanics::targets::is_nonland_permanent_card;
use cantrip_mechanics::{choose, library, top_of_library};
use cantrip_oracle as oracle;

/// "Noncreature, nonland permanent cards with mana value 3 or less" (CR 110.4b, 202.3e): judged on the card in the library,
/// where an {X} in its cost counts as 0, which `mana_value` already says.
fn eligible(g: &dyn Game, card: ObjectId) -> bool {
    let chars = g.chars(card);
    is_nonland_permanent_card(&chars) && !chars.is(CardType::Creature) && g.mana_value(card) <= 3
}

/// "Look at the top seven cards of your library. Put up to two noncreature, nonland permanent cards with mana value 3 or
/// less from among them onto the battlefield. Put the rest on the bottom of your library in a random order." The two enter
/// together (CR 110.2a); a library of fewer than seven shows what it has (CR 609.3), and with no eligible card the player
/// has looked at the cards (`look_at`) but is asked nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let looked = top_of_library(g, you, 7);
    if looked.is_empty() {
        return Ok(());
    }
    g.look_at(you, &looked);
    let permitted: Vec<ObjectId> = looked.iter().copied().filter(|&c| eligible(g, c)).collect();
    let chosen = choose::cards_from_kind(
        g,
        you,
        item.object,
        Purpose::PutOntoBattlefield,
        "noncreature, nonland permanent with mana value 3 or less",
        looked.clone(),
        permitted,
        2,
        true,
    )?;
    if !chosen.is_empty() {
        let entering: Vec<(ObjectId, PlayerId)> = chosen.into_iter().map(|c| (c, you)).collect();
        g.put_all_onto_battlefield(&entering)?;
    }
    library::rest_to_bottom_in_random_order(g, you, &looked);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::UNITED_BATTLEFRONT
};
