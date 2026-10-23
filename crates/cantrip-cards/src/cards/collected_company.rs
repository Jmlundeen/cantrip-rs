use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_oracle as oracle;

/// "Creature cards with mana value 3 or less" (CR 205.2a, 202.3e): each card's own mana value, read in the library
/// where it is, not their total.
fn eligible(q: &dyn Query, card: ObjectId) -> bool {
    q.chars(card).is(CardType::Creature) && q.mana_value(card) <= 3
}

/// "Look at the top six cards of your library. Put up to two creature cards with mana value 3 or less from among them onto
/// the battlefield. Put the rest on the bottom of your library in any order." They enter under your control (CR 110.2a); the rest
/// go to the bottom in the order you choose (CR 401.4); a library of fewer than six shows what it has (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield(g, item.controller, item.object, 6, 2, "creature with mana value 3 or less", eligible, None, library::Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::COLLECTED_COMPANY };
