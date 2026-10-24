use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_mechanics::library;
use cantrip_mechanics::targets::is_permanent_card;
use cantrip_oracle as oracle;

/// "A permanent card with mana value 5 or less" (CR 110.4, 202.3e): judged on the card in the library, where an {X} in its
/// cost counts as 0, which `mana_value` already says.
fn eligible(q: &dyn Query, card: ObjectId) -> bool {
    is_permanent_card(&q.chars(card)) && q.mana_value(card) <= 5
}

/// "Look at the top five cards of your library. You may put up to two permanent cards with mana value 5 or less from among
/// them onto the battlefield. Put the rest on the bottom of your library in a random order." The two enter
/// together; a library of fewer than five shows what it has (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 5, 2, "permanent with mana value 5 or less", eligible, None, false, library::Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{7}{G}{G}{G}"), "Flashback {7}{G}{G}{G}")],
    ..oracle::STORM_THE_FESTIVAL
};
