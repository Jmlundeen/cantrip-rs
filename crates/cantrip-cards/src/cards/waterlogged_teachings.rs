use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Search your library for an instant card or a card with flash, reveal it, put it into your hand, then shuffle. The
/// search names a quality, so the searcher may find nothing (CR 701.23b); the card is revealed while it is still in
/// the library, and the library is shuffled either way.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Instant) || chars.has(Keyword::Flash)
    })?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Blue, Color::Black]), "{T}: Add {U} or {B}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::INUNDATED_ARCHIVE
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    back: Some(&BACK),
    ..oracle::WATERLOGGED_TEACHINGS
};
