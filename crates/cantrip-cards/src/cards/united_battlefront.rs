use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::targets::is_nonland_permanent_card;
use cantrip_oracle as oracle;

/// "Noncreature, nonland permanent cards with mana value 3 or less" (CR 110.4b, 202.3e): judged on the card in the library,
/// where an {X} in its cost counts as 0, which `mana_value` already says.
fn eligible(q: &dyn Query, card: ObjectId) -> bool {
    let chars = q.chars(card);
    is_nonland_permanent_card(&chars) && !chars.is(CardType::Creature) && q.mana_value(card) <= 3
}

/// "Look at the top seven cards of your library. Put up to two noncreature, nonland permanent cards with mana value 3 or
/// less from among them onto the battlefield. Put the rest on the bottom of your library in a random order." The two enter
/// together (CR 110.2a); a library of fewer than seven shows what it has (CR 609.3), and with no eligible card the player
/// has looked at the cards but is asked nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let kind = "noncreature, nonland permanent with mana value 3 or less";
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 7, 2, kind, eligible, None, false, library::Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::UNITED_BATTLEFRONT
};
