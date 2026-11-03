use cantrip_api::*;
use cantrip_mechanics::{discover, flashback, library, permanents_with_subtype_you_control};
use cantrip_oracle as oracle;

/// "Exile cards from the top of your library until you exile a nonland card. You may cast that card without paying its mana cost
/// if the spell's mana value is less than the number of Mountains you control. If you don't cast that card this way, put it into
/// your hand." The Mountains are counted as the spell resolves, by the subtype Mountain as a word of this card's text (CR 612);
/// "less than" is "at most one less" - and with no Mountains nothing is less, so the card goes to the hand. The lands exiled on the
/// way stay in exile. The spell's mana value is that of the face it is cast as (CR 601.2b), as Discover reads it (CR 701.57a).
fn exile_and_cast_or_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let (_, hit) = library::exile_until_nonland(g, you, None, &|_, _| true);
    let Some(card) = hit else { return Ok(()) };
    let mountain = g.words(item.object).subtype(Subtype::Mountain);
    let mountains = permanents_with_subtype_you_control(g, you, mountain) as u32;
    match mountains.checked_sub(1) {
        Some(most) => discover::cast_free_if_at_most_else_hand(g, you, card, most)?,
        None => {
            g.return_to_hand(&[card]);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], exile_and_cast_or_take),
    ways: &[flashback::way(cost("{6}{R}"), "Flashback {6}{R} (You may cast this card from your graveyard for its flashback cost. Then exile it.)")],
    ..oracle::SOLSTICE_REVELATIONS
};
